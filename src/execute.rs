use std::collections::VecDeque;
use std::fs::{self};
use std::io::{self, BufRead, BufReader, BufWriter, Lines, Read, Write, stderr};
use std::iter::{self, Peekable, chain, once};
use std::ops::{ControlFlow, Deref, Index};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use std::path::{self, Path, PathBuf};
use std::process::{self, Output, Stdio};
use std::{array, mem, vec};
use std::ffi::OsString;

use tempfile::{NamedTempFile, TempDir, TempPath};
use uuid::Uuid;

use crate::args::Args;
use crate::args::io::{FileOrStream, Out};
use crate::parse::{Block, BlockMarkers, File, Loc, MarkerInst, Span};

#[derive(Debug, Clone)]
struct OutputTerminator(String);

impl OutputTerminator {
  fn new() -> Self {
    Self(Uuid::new_v4().into())
  }
}

impl Deref for OutputTerminator {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FileExecutor<'a> {
    pub file: &'a File<'a>,
    cmd: process::Command,
    output_terminators: Vec<OutputTerminator>,
    temp_dir: TempDir,
}

impl<'a> FileExecutor<'a> {
    /// Construct a command
    pub fn new(file: &'a File<'a>) -> io::Result<FileExecutor<'a>> {
        let source_ext = match file.source.to_str().rsplit_once('.') {
            Some((_, ext)) => ext,
            None => "",
        };
        let temp_dir = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
        let temp_path = temp_dir.path();

        let mut env: Vec<(String, &OsStr)> = vec![];

        macro_rules! var {
            ($namefmt:literal => $val:expr) => {
                env.push((format!("COGSH_{}", $namefmt), $val));
            };
            ($namefmt:literal [$i:expr] => $val:expr) => {
                env.push((format!("COGSH_{}_{}", $namefmt, $i), $val));
            };
        }
        macro_rules! path {
            ($fname:expr, $content:expr) => {{
                let path = temp_path.join($fname);
                fs::write(&path, $content)?;
                path
            }};
        }

        // nonces to figure out where output from one block ends and the next begins
        let output_terminators: Vec<_> = iter::repeat_with(OutputTerminator::new)
            .take(file.blocks.len() + 1)
            .collect();

        var!("TEMP_DIR" => temp_path.as_os_str());
        var!("SOURCE" => OsStrExt::from_bytes(file.source.to_str().as_bytes()) );
        var!("NUM_BLOCKS" => file.blocks.len().to_string().as_bytes());
        var!("PROLOGUE" => path!("prologue.sh", file.config.prologue.join("\n")));

        for (i0, block) in file.blocks.iter().enumerate() {
            let i1 = i0 + 1; // 1-based indexing for var names

            let span = block.markers.prog_beg.span;
            var!("BLOCK_LINE" [i1] => span.start.line);
            var!("BLOCK_COL" [i1] => span.start.col);
            var!("BLOCK_OFFSET" [i1] => span.start.offset);

            var!("BLOCK_PROG" [i1] => path!(format!("block_{i1}.sh"), block.prog_lines.join("\n")));
            var!("BLOCK_OUTPUT_LINE_PFX" [i1] => block.prog_whitespace_pfx);
            var!("BLOCK_OUTPUT_PREV" [i1] => path!(format!("output_prev_{i1}{source_ext}"), block.output_prev));
        }

        let mut cmd = process::Command::new("bash");
        cmd.args(["-c", include_str!("program.sh")]);
        cmd.arg(&file.source.to_str()); // make $COGSH_SOURCE also available as $0
        cmd.args(output_terminators); // pass output terminators as $1..$N rather than in env to reduce visibility to block code
        cmd.envs(env);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        // set current directory to parent dir of source file (if source is not stdin)
        if let FileOrStream::File(input_path, _) = &file.source {
            let input_path_abs = path::absolute(input_path)?;
            cmd.current_dir(input_path_abs.parent().unwrap());
        }

        Ok(FileExecutor {
            file,
            cmd,
            output_terminators,
            temp_dir,
        })
    }

    pub fn execute_in_place(&mut self) -> io::Result<()> {
        match &self.file.source {
            FileOrStream::File(orig_path, _) => {
                let temp_path = NamedTempFile::new_in(orig_path.parent().unwrap())?
                    .into_temp_path()
                    .to_path_buf();
                let output = FileOrStream::file(temp_path.clone());
                self.execute(output)?;
                fs::rename(temp_path, orig_path)?;
            }
            FileOrStream::Stream(_) => {
                self.execute(FileOrStream::stream())?;
            }
        };
        Ok(())
    }

    pub fn execute(&mut self, output: FileOrStream<Out>) -> io::Result<()> {
        let mut outp_writer = output.open()?;

        let proc = self.cmd.spawn()?;
        let mut proc_reader = BufReader::new(proc.stdout.unwrap());

        macro_rules! nonce_terminated {
            (until $term:expr, for $line:ident in $reader:expr, $body:expr) => {
                let mut curr_line: ByteString = vec![];
                let mut next_line: ByteString = vec![];
                loop {
                    mem::swap(&mut curr_line, &mut next_line); // avoid re-allocating vectors
                    next_line.clear();
                    match $reader.read_until(b'\n', &mut next_line)? {
                        0 => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                        _ => {
                            if next_line.split_last() == Some((&b'\n', $term.as_bytes())) {
                                assert_eq!(curr_line.pop(), Some(b'\n'));
                                let $line = &curr_line[..];
                                $body;
                                break;
                            } else {
                                let $line = &curr_line[..];
                                $body;
                            }
                        }
                    }
                }
            };
        }

        let mut stderr = io::stderr();

        nonce_terminated! {
          until self.output_terminators[0], for prologue_line in proc_reader, {
            stderr.write_all(format!("[PROLOGUE {}] ", self.file.source.to_str()).as_bytes())?;
            stderr.write_all(prologue_line)?;
          }
        }

        // portion of the file before the first block
        let head = &self.file.content[..*self.file.blocks[0].span.start];
        outp_writer.write(head);

        for i in self.blocks
        nonce_terminated! {
          until self.output_terminators[0], for prologue_line in proc_reader, {
            stderr.write_all(format!("[PROLOGUE {}] ", self.file.source.to_str()).as_bytes())?;
            stderr.write_all(prologue_line)?;
          }
        }

        Ok(())
        //   match line {
        //     Err(e) => Some(Err(e)),
        //     Ok(0) => {
        //       let err = io::Error::from(io::ErrorKind::UnexpectedEof);
        //       Some(Err(err))
        //     },
        //     Ok(_) if next_line[..] == terminator.as_bytes() => {
        //       None
        //     },
        //     Ok(_) => {
        //       Some(Ok(&curr_line))
        //     }
        //   }
        // });
    }
}

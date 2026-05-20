use std::fs::{self};
use std::io::{self, BufRead, BufReader, Write};
use std::iter::{self, repeat_with};
use std::ops::Deref;

use std::path::{self};
use std::process::{self, Stdio};
use std::{mem, vec};

use tempfile::TempDir;

use crate::args::{Pipe, PipeDir};
use crate::parse::File;

#[derive(Debug, Clone)]
struct OutputTerminator(String);

impl OutputTerminator {
    fn new() -> Self {
        Self(repeat_with(fastrand::alphanumeric).take(20).collect())
    }
}

impl Deref for OutputTerminator {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct FileExecutor<'a> {
    pub file: &'a File<'a>,
    cmd: process::Command,
    output_terminators: Vec<OutputTerminator>,
    #[allow(dead_code)]
    temp_dir: TempDir,
}

impl<'a> FileExecutor<'a> {
    /// Construct a command
    pub fn new(file: &'a File<'a>) -> io::Result<FileExecutor<'a>> {
        let source = file.source.with_dir(PipeDir::Read).to_string();
        let source_ext = source.rsplit_terminator('.').next().unwrap_or("");
        let temp_dir = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
        let temp_path = temp_dir.path();

        let mut env = vec![];

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
                path.to_str().unwrap().to_owned()
            }};
        }

        // nonces to figure out where output from one block ends and the next begins
        let output_terminators: Vec<_> = iter::repeat_with(OutputTerminator::new)
            .take(file.blocks.len() + 1)
            .collect();

        var!("TEMP_DIR" => temp_path.to_str().unwrap().to_owned());
        var!("SOURCE" => source.clone());
        var!("NUM_BLOCKS" => file.blocks.len().to_string());
        var!("PROLOGUE" => path!("prologue.sh", file.config.prologue.join("\n")));

        for (i0, block) in file.blocks.iter().enumerate() {
            let i1 = i0 + 1; // 1-based indexing for var names

            let span = block.markers.prog_beg.span;
            var!("BLOCK_LINE" [i1] => span.start.line.to_string());
            var!("BLOCK_COL" [i1] => span.start.col.to_string());
            var!("BLOCK_OFFSET" [i1] => span.start.offset.to_string());

            var!("BLOCK_PROG" [i1] => path!(format!("block_{i1}.sh"), block.prog_lines.join("\n")));
            var!("BLOCK_OUTPUT_LINE_PFX" [i1] => block.prog_whitespace_pfx.to_owned());
            var!("BLOCK_OUTPUT_PREV" [i1] => path!(format!("output_prev_{i1}{source_ext}"), block.output_prev));
        }

        let mut cmd = process::Command::new("bash");
        cmd.args(["-c", include_str!("program.sh")]);
        cmd.arg(&source.clone()); // make $COGSH_SOURCE also available as $0
        for term in &output_terminators {
            cmd.arg(&*term as &str);
        }
        cmd.envs(env);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        // set current directory to parent dir of source file (if source is not stdin)
        if let Pipe::File(input_path) = &file.source {
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

    pub fn execute(&mut self, output: &Pipe) -> io::Result<()> {
        let mut outp_writer = output.open_for_write()?;

        let proc = self.cmd.spawn()?;
        let mut proc_reader = BufReader::new(proc.stdout.unwrap());

        macro_rules! nonce_terminated {
            (until $term:expr, for $line:ident in $reader:expr, $body:expr) => {
                let mut curr_line = String::new();
                let mut next_line = String::new();
                loop {
                    mem::swap(&mut curr_line, &mut next_line); // avoid re-allocating vectors
                    next_line.clear();
                    match $reader.read_line(&mut next_line)? {
                        0 => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                        _ => {
                            if next_line.strip_suffix("\n") == Some($term) {
                                assert_eq!(curr_line.pop(), Some('\n'));
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

        // write each line of output until terminator to output with the given prefix
        macro_rules! tee_wrapped {
            (to: $to_output:expr, prefix: $prefix:expr, until: $terminator:expr) => {
                let output = &mut $to_output;
                nonce_terminated! {
                  until $terminator, for line in proc_reader, {
                    output.write_all($prefix.as_bytes())?;
                    output.write_all(line.as_bytes())?;
                  }
                }
            };
        }

        // write any output of prologue to stderr
        tee_wrapped!(
          to: io::stderr(),
          prefix: format!("[PROLOGUE {}] ", self.file.source.with_dir(PipeDir::Read)),
          until: &*self.output_terminators[0]
        );

        // portion of the file before the first block
        let head = &self.file.content[..*self.file.blocks[0].span.start];
        outp_writer.write_all(head.as_bytes())?;

        // for each block
        for i in 0..self.file.blocks.len() {
            let block = &self.file.blocks[i];
            let terminator = &*self.output_terminators[i + 1];

            // write output of block itself
            tee_wrapped!(
              to: outp_writer,
              prefix: block.prog_whitespace_pfx,
              until: terminator
            );

            // write portion of file between block and next block (or EOF)
            let block_tail = {
                let next_block_start_or_eof = match self.file.blocks.get(i + 1) {
                    Some(next_block) => *next_block.span.start,
                    None => self.file.content.len(),
                };
                &self.file.content[*block.span.end..next_block_start_or_eof]
            };
            outp_writer.write_all(block_tail.as_bytes())?;
        }

        Ok(())
    }
}

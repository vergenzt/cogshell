use std::fs::{self};
use std::io::{self, BufRead as _, BufReader, BufWriter, Write, stderr};
use std::iter::{self};
use std::path::{self, Path, PathBuf};
use std::process::{self, Stdio};
use std::{array, vec};

use tempfile::TempDir;
use uuid::Uuid;

use crate::args::Args;
use crate::args::io::{FileOrStream, Out};
use crate::parse::{Block, BlockMarkers, File, Loc, MarkerInst, Span};

pub fn execute(file: &File, output: &FileOrStream<Out>) -> io::Result<()> {
    let executor = FileExecutor::new(file);
}

pub struct FileExecutor<'a> {
    pub file: &'a File<'a>,
    cmd: process::Command,
    output_terminators: Vec<String>,
    temp_dir: TempDir,
}

impl<'a> FileExecutor<'a> {
    /// Construct a command
    pub fn new(file: &'a File<'a>) -> io::Result<FileExecutor<'a>> {
        let source_ext = match file.source_name().rsplit_once('.') {
            Some((_, ext)) => ext,
            None => "",
        };
        let temp_dir = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
        let temp_path = temp_dir.path();

        let mut env: Vec<(String, String)> = vec![];

        macro_rules! var {
            ($namefmt:literal => $val:expr) => {
                env.push((format!("COGSH_{}", $namefmt), $val.to_string()));
            };
            ($namefmt:literal [$i:expr] => $val:expr) => {
                env.push((format!("COGSH_{}_{}", $namefmt, $i), $val.to_string()));
            };
        }
        macro_rules! path {
            ($fname:expr, $content:expr) => {{
                let path = temp_path.join($fname);
                fs::write(&path, $content)?;
                path.display().to_string()
            }};
        }

        // nonces to figure out where output from one block ends and the next begins
        let output_terminators: Vec<_> = iter::repeat_with(|| format!("{}\n", Uuid::new_v4()))
            .take(file.blocks.len() + 1)
            .collect();

        var!("TEMP_DIR" => temp_path.display());
        var!("SOURCE" => file.source_name());
        var!("NUM_BLOCKS" => file.blocks.len());
        var!("PROLOGUE" => path!("prologue.sh", file.config.prologue.join("\n")));
        var!("PROLOGUE_TERMINATOR" => output_terminators[0]);

        for (i0, block) in file.blocks.iter().enumerate() {
            let i1 = i0 + 1; // 1-based indexing for var names

            let span = block.markers.prog_beg.span;
            var!("BLOCK_LINE" [i1] => span.start.line);
            var!("BLOCK_COL" [i1] => span.start.col);
            var!("BLOCK_OFFSET" [i1] => span.start.offset);

            var!("BLOCK_PROG" [i1] => path!(format!("block_{i1}.sh"), block.prog_lines.join("\n")));
            var!("BLOCK_OUTPUT_LINE_PFX" [i1] => block.prog_whitespace_pfx);
            var!("BLOCK_OUTPUT_PREV" [i1] => path!(format!("output_prev_{i1}{source_ext}"), block.output_prev));

            let output_terminator: &str = match output_terminators.get(i1) {
                Some(s) => &s,
                None => "", // final terminator is empty (will be EOF)
            };
            var!("BLOCK_OUTPUT_TERMINATOR" [i1] => output_terminator);
        }

        let mut cmd = process::Command::new("bash");
        cmd.args(["-c", include_str!("program.sh")]);
        cmd.arg(&file.source_name()); // make $COGSH_SOURCE also available as $0
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

    pub fn execute_replace(&mut self) -> io::Result<()> {}

    pub fn execute(&mut self, output: FileOrStream<Out>) -> io::Result<()> {
        let File { ctx, blocks } = self.file;
        let proc = self.cmd.spawn()?;

        let mut output_reader = BufReader::new(proc.stdout.unwrap());
        let mut output_buf = String::new();

        let mut loc: Loc = Loc::default();
        let mut out: Box<dyn Write> = Box::new(stderr());
        let mut pfx: &str = &format!("[PROLOGUE {}] ", self.file.source_name());

        for i in 0..=blocks.len() {
            let block_opt = blocks.get(i - 1);
            if let Some(block) = block_opt {
                out = Box::new(output_writer);
                pfx = block.prog_whitespace_pfx;

                let BlockMarkers {
                    prog_beg,
                    prog_end,
                    outp_end,
                } = block.markers;

                // output previous content up to end of this block's program
                write!(out, "{}", &file.content[*loc..*prog_end.span.end]);

                // add newline if there's a newline between output markers
                if prog_end.span.line() != outp_end.span.line() {
                    writeln!(out, "");
                }
            }

            let terminator = &output_terminators[i];

            loop {
                let n = output_reader.read_line(&mut output_buf)?;

                // EOF before an expected terminator is an error
                if n == 0 {
                    let desc = match block_opt {
                        None => "prologue",
                        Some(block) => &format!("block at {}:{}", source_name, block.span.start),
                    };
                    return Err(io::Error::other(format!(
                        "unexpected EOF while waiting for end of {desc}"
                    )));
                }

                let last_line = &output_buf[output_buf.len() - n..];

                // if last_line ends with terminator...
                // (NB: terminator_pfx may be empty string)
                if let Some(part_before_terminator) = last_line.strip_suffix(terminator) {
                    // if prologue or block output doesn't end with a newline then there will be a prefix to the terminator
                    // which needs to be output.
                    if part_before_terminator != "" {
                        // check whether to add an extra newline to separate from subsequent output
                        let extra_newline = match block_opt {
                            // prologue output => add newline if it doesn't end with one
                            None => "\n",
                            // block output with colinear output delimiters => no newline
                            Some(Block { markers, .. })
                                if markers.prog_end.span.line() == markers.outp_end.span.line() =>
                            {
                                ""
                            }
                            // anything else
                            _ => "",
                        };
                        write!(out, "{}{}{}", pfx, part_before_terminator, extra_newline)?;
                    }

                    if let Some(block) = block_opt {
                        write!(out, "{}{}", pfx, block.markers.outp_end.str())?;
                    }

                    // always break inner line-output loop on termination
                    break;
                } else {
                    write!(out, "{}{}", pfx, last_line)?;
                }
            }
        }

        let block_suffix = match blocks.get(i1) {
            Some(next_block) => &file.content[block.end()..next_block.start()],
            None => &file.content[block.end()..],
        };
        var!("BLOCK_SUFFIX" [i1] => path!(format!("block_suffix_{i1}{source_ext}"), block_suffix));
    }
}

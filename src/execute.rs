use std::fs::{self};
use std::io::{self, BufRead as _, BufReader, BufWriter, Write, stderr};
use std::iter::{self};
use std::path::PathBuf;
use std::process::{self, Stdio};
use std::{array, vec};

use uuid::Uuid;

use crate::args::Args;
use crate::args::io::{FileOrStream, Out};
use crate::parse::{BlockMarkers, File, Loc, MarkerInst, Span};

pub fn execute(file: &File, output: &FileOrStream<Out>) -> io::Result<()> {
    let File { ctx, blocks } = file;
    let source_name = ctx.input.to_string();
    let source_ext = match source_name.rsplit_once('.') {
        Some((_, ext)) => ext,
        None => "",
    };
    let temp_dir_obj = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
    let temp_dir = temp_dir_obj.path();

    let mut env: Vec<(String, String)> = vec![];

    macro_rules! var {
        ($namefmt:literal => $val:expr) => {
            env.push((format!("COGSH_{}", $namefmt), $val.to_string()));
        };
        ($namefmt:literal $i:ident => $val:expr) => {
            env.push((format!("COGSH_{}_{}", $namefmt, $i), $val.to_string()));
        };
    }
    macro_rules! path {
        ($fname:expr, $content:expr) => {{
            let path = temp_dir.join($fname);
            fs::write(&path, $content)?;
            path.display().to_string()
        }};
    }

    // nonces to figure out where output from one block ends and the next begins
    let output_terminators: Vec<_> = iter::repeat_with(|| format!("\n{}\n", Uuid::new_v4()))
        .take(blocks.len())
        .collect();

    var!("TEMP_DIR" => temp_dir.display());
    var!("SOURCE" => source_name);
    var!("NUM_BLOCKS" => blocks.len());
    var!("PROLOGUE" => path!("prologue.sh", ctx.config.prologue.join("\n")));
    var!("PROLOGUE_TERMINATOR" => output_terminators[0]);

    for (i0, block) in blocks.iter().enumerate() {
        let i1 = i0 + 1; // 1-based indexing

        let span = block.markers.prog_beg.span;
        var!("BLOCK_LINE" i1 => span.start.line);
        var!("BLOCK_COL" i1 => span.start.col);
        var!("BLOCK_OFFSET" i1 => span.start.offset);

        var!("BLOCK_PROG" i1 => path!(format!("block_{i1}.sh"), block.prog_lines.join("\n")));
        var!("BLOCK_OUTPUT_LINE_PFX" i1 => block.prog_whitespace_pfx);
        var!("BLOCK_OUTPUT_PREV" i1 => path!(format!("output_prev_{i1}{source_ext}"), block.output_prev));

        let output_terminator: &str = match output_terminators.get(i1) {
            Some(s) => &s,
            None => "", // final terminator is empty (will be EOF)
        };
        var!("BLOCK_OUTPUT_TERMINATOR" i1 => output_terminator);
    }

    let mut output_writer = BufWriter::new(output.open()?);

    let proc = {
        let mut cmd = process::Command::new("bash");
        cmd.args(["-c", include_str!("program.sh")]);
        cmd.arg(&source_name); // make $COGSH_SOURCE also available as $0
        cmd.envs(env);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());
        cmd.spawn()?
    };

    let mut output_reader = BufReader::new(proc.stdout.unwrap());
    let mut output_buf = String::new();

    let mut loc: Loc = Loc::default();
    let mut out: Box<dyn Write> = Box::new(stderr());
    let mut pfx: &str = &format!("[PROLOGUE {}] ", source_name);

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
            write!(out, "{}", &ctx.content[*loc..*prog_end.span.end]);

            // add newline if there's a newline between output markers
            if prog_end.span.line() != outp_end.span.line() {
                writeln!(out, "");
            }
        }

        loop {
            match output_reader.read_line(&mut output_buf)? {
                0 /* EOF */ => {
                    let desc = match block_opt {
                        None => "prologue",
                        Some(block) => &format!("block at {}:{}", source_name, block.span.start),
                    };
                    return Err(io::Error::other(format!(
                        "unexpected end of file while waiting for end of {desc}"
                    )));
                }
                _ if output_buf.ends_with(&output_terminators[i]) => {
                    break;
                }
                n => {
                    let last_line = &output_buf[output_buf.len() - n..];
                    write!(out, "{}{}", pfx, last_line);
                }
            }

            if let Some(block) = block_opt {
                write!(out, "{}", block.markers.outp_end.str());

                match ctx.config.
            }
        }
    }

    let block_suffix = match blocks.get(i1) {
        Some(next_block) => &ctx.content[block.end()..next_block.start()],
        None => &ctx.content[block.end()..],
    };
    var!("BLOCK_SUFFIX" i1 => path!(format!("block_suffix_{i1}{source_ext}"), block_suffix));
}

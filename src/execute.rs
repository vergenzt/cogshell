use std::fs::{self, File};
use std::io::{self, BufRead as _, BufReader, Write};
use std::iter::{self};
use std::path::PathBuf;
use std::process::{self, Stdio};

use uuid::Uuid;

use crate::parse::{BlockMarkers, File, MarkerInst};

pub fn execute(&File { file: ctx, blocks }: &File) -> io::Result<()> {
    let file_path = PathBuf::from(ctx.filename);
    let file_ext = ctx
        .filename
        .rfind('.')
        .filter(|&i| i > 0)
        .map(|i| &ctx.filename[i..])
        .unwrap_or("");
    let temp_dir_obj = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
    let temp_dir = temp_dir_obj.path();

    let vars = &ctx.config.env_var_names;
    let program_path = temp_dir.join("program.sh");
    let mut prg = File::create_buffered(&program_path)?;
    let mut prg_env = vec![];

    writeln!(prg, "#!/usr/bin/env bash")?;

    prg_env.push((vars.temp_dir, temp_dir.to_str().unwrap()));
    prg_env.push((vars.source_path, ctx.filename));

    // create nonces to separate blocks (incl. prologue)
    // nonce 0 terminates the prologue, remainder terminate blocks except the last
    // NB: each nonce val begins and ends with a newline.
    let output_sep_nonces: Vec<String> = iter::repeat_with(|| format!("\n{}\n", Uuid::new_v4()))
        .take(blocks.len())
        .collect();

    // push nonces into environment
    for (i, nonce) in output_sep_nonces.iter().enumerate() {
        prg_env.push((&format!("{}_{}", vars.output_sep_nonce, i), nonce));
    }

    macro_rules! var {
        ($name:ident) => {
            var!(vars.$name)
        };
        ($name:ident, $sfx:expr) => {
            var!(format!("{}_{}", vars.$name, $sfx))
        };
        ($expr:expr) => {
            format!("\"${{{}}}\"", $expr)
        };
    }
    macro_rules! make_file {
        ($name:expr, $content:expr) => {
            fs::write(temp_dir.join($name), $content).and(Ok(format!(
                "{}/{}",
                var!(temp_dir),
                $name
            )))?
        };
    }
    macro_rules! source {
        ($name:expr) => {
            writeln!(prg, "source {}/{}", var!(temp_dir), $name)?
        };
    }
    macro_rules! write_nonce {
        ($n:expr) => {
            writeln!(prg, "echo {}", var!(output_sep_nonce, $n))?
        };
    }

    // write prologue
    writeln!(prg, "echo 'executing prologue for {}...' >&2", ctx.filename)?;
    source!(make_file!("prologue.sh", &ctx.config.prologue.join("\n")));
    write_nonce!(0);

    // write blocks
    for (i, block) in blocks.iter().enumerate() {
        let output_prev = make_file!(
            &format!("output_prev.{}{}", i + 1, file_ext),
            block.output_prev
        );
        writeln!(prg, "export {}={}", vars.output_prev, output_prev)?;

        let MarkerInst {
            r#match: span,
            line,
            col,
        } = block.markers[0];
        writeln!(prg, "export {}={}", vars.prog_start_line, line)?;
        writeln!(prg, "export {}={}", vars.prog_start_col, col)?;
        writeln!(prg, "export {}={}", vars.prog_start_offset, span.start())?;

        let source_ref = format!("{}:{}:{}", var!(source_path), line, col);
        writeln!(prg, "echo 'executing block at {source_ref}...' >&2")?;

        source!(make_file!(
            format!("program_{}.sh", i + 1),
            block.prog_lines.join("\n")
        ));
        write_nonce!(i + 1);
    }
    prg.flush()?;

    let proc = process::Command::new("bash")
        .arg(&program_path)
        .envs(prg_env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let output_path = temp_dir.join("output").with_extension(file_ext);
    let mut output_file = File::create_buffered(output_path)?;
    let mut output_reader = BufReader::new(proc.stdout.unwrap());
    let mut output_buf = String::new();

    macro_rules! for_each_line_until {
        ($desc:expr, $nonce:expr, $handle_line:expr) => {{
            output_buf.clear();
            loop {
                let n = output_reader.read_line(&mut output_buf)?;
                if n == 0 {
                    return Err(io::Error::other(format!(
                        "unexpected EOF while waiting for termination of {}",
                        $desc
                    )));
                }

                if output_buf.ends_with($nonce) {
                    break;
                } else {
                    let last_line = &output_buf[output_buf.len() - n..];
                    $handle_line(last_line)?;
                }
            }
        }};
    }

    let prologue_nonce = &output_sep_nonces[0];
    for_each_line_until!("prologue", prologue_nonce, |prologue_output_line| {
        // forward prologue output to terminal
        print!("{}", prologue_output_line);
        Ok(())
    });

    for i0 in 0..blocks.len() {
        let block = blocks[i0];
        let BlockMarkers([prog_start, prog_end, outp_end]) = block.markers;

        let block_pfx = {
            let pfx_end = prog_end.r#match.end();
            let pfx_start = blocks
                .get(i0 - 1)
                .map(|prev_block| prev_block.markers.outp_end().r#match.end())
                .unwrap_or(0);
            &ctx.content[pfx_start..pfx_end]
        };
        write!(output_file, "{block_pfx}")?;

        let i1 = i0 + 1;
        let block_nonce = &output_sep_nonces[i1];
        let block_ws_pfx = block.prog_whitespace_pfx;

        let wrap_output = prog_end.line == outp_end.line;
        if wrap_output {
            writeln!(output_file, "");
        }

        for_each_line_until!(
            format!("block {i1}"),
            block_nonce,
            |block_outp_line| -> io::Result<()> {
                if block_outp_line != "\n" {
                    write!(output_file, "{block_ws_pfx}")?;
                }
                write!(output_file, "{block_outp_line}")?;
                Ok(())
            }
        );

        if wrap_output {
            writeln!(output_file, "");
        }

        write!(output_file, "{}", outp_end.r#match.as_str());

        // copy remainder of file if this is the last block
        if i1 == blocks.len() {
            let sfx = &ctx.content[outp_end.r#match.end()..];
            write!(output_file, "{sfx}");
        }
    }

    Ok(())
}

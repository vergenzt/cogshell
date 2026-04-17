use std::fs::{self};
use std::io::{self, BufRead as _, BufReader, Write};
use std::iter::{self};
use std::path::PathBuf;
use std::process::{self, Stdio};

use uuid::Uuid;

use crate::args::io::FileOrStream;
use crate::config::EnvConfig;
use crate::parse::{BlockMarkers, File, MarkerInst};

pub fn execute(file: &File) -> io::Result<()> {
    let File { ctx, mut blocks } = file;
    let file_ext = match ctx.input {
        FileOrStream::File(path, _) => path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or_default(),
        _ => "",
    };
    let temp_dir_obj = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
    let temp_dir = temp_dir_obj.path();

    let EnvConfig {
        source_path_var,
        num_blocks_var,
        temp_dir_var,
        prog_start_line_var,
        prog_start_col_var,
        prog_start_offset_var,
        output_prev_var,
        output_sep_nonce_var,
    } = ctx.config.env_var_names;

    let program_path = temp_dir.join("program.sh");
    let mut prg = fs::File::create_buffered(&program_path)?;
    let mut prg_env = vec![];

    prg_env.push((temp_dir_var, temp_dir.to_str().unwrap()));
    prg_env.push((source_path_var, &ctx.input.to_string()));
    prg_env.push((num_blocks_var, &blocks.len().to_string()));

    for (i, block) in blocks.iter().enumerate() {
        let block_start = block.markers.prog_start().span;
        let block_var_vals: &[(String, &str)] = &[
            (prog_start_line_var, &block_start.line.to_string()),
            (prog_start_col_var, &block_start.col.to_string()),
            (prog_start_offset_var, &block_start.start.to_string()),
            (output_prev_var, block.output_prev),
            (output_sep_nonce_var, &format!("\n{}\n", Uuid::new_v4())),
        ];
        for (var, val) in block_var_vals {
            prg_env.push((format!("{}_{}", var, i), val));
        }
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

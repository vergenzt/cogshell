mod errors;

pub use errors::*;

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufRead as _, BufReader, Write};
use std::iter::{self, chain, once, zip};
use std::path::{self, PathBuf};
use std::process::{self, Stdio};

use uuid::Uuid;

use crate::parse::{Marker, MarkerKind, ParsedBlock, ParsedFile};

pub fn execute(&ParsedFile { ctx, blocks }: &ParsedFile) -> io::Result<()> {
    let file_path = PathBuf::from(ctx.filename);
    let file_ext: OsString = match file_path.extension() {
        Some(ext) => ext.into(),
        None => "".into(),
    };
    let temp_dir = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
    let temp_dir_path = temp_dir.path();

    // create nonces to figure out where output of one block ends and another begins
    // NB: each nonce val begins and ends with a newline
    let output_sep_nonces: Vec<String> = {
        // one for each block plus one for the prologue
        let n = blocks.len() + 1;
        iter::repeat_with(|| format!("\n{}\n", Uuid::new_v4()))
            .take(n)
            .collect()
    };

    let vars = &ctx.config.env_var_names;
    let program_path = temp_dir_path.join("program.sh");
    let mut prg = File::create_buffered(&program_path)?;
    let mut prg_args = vec![];

    writeln!(prg, "#!/usr/bin/env bash")?;
    writeln!(prg, "export {}=${}", vars.source_file_path, {
        prg_args.push(ctx.filename);
        prg_args.len()
    });

    // write prologue
    {
        let prologue_path = temp_dir_path.join("prologue.sh");
        fs::write(prologue_path, ctx.config.prologue.join("\n"));
        writeln!(prg, "source {}", prologue_path.display());
    }

    for i in 0..blocks.len() + 1 {
        let prog_lines = if i == 0 {
            writeln!(prg, "echo 'executing prologue for {}...' >&2", ctx.filename);
            &ctx.config.prologue
        } else {
            let block = &blocks[i - 1];

            let path = temp_dir_path
                .join("output_prev")
                .with_added_extension((i + 1).to_string())
                .with_added_extension(&file_ext);
            fs::write(&path, &block.output_prev)?;
            writeln!(
                prg,
                "export {}=${{{}}}/{}",
                vars.output_prev,
                vars.tmp_dir,
                path.strip_prefix(temp_dir_path).unwrap().display()
            )?;

            let Marker { span, line, col } = block.markers[0];
            writeln!(prg, "export {}={}", vars.prog_start_line, line)?;
            writeln!(prg, "export {}={}", vars.prog_start_col, col)?;
            writeln!(prg, "export {}={}", vars.prog_start_offset, span.start())?;
            writeln!(
                prg,
                "echo 'executing block at ${{{}}}:{}:{}...' >&2",
                vars.source_file_path, line, col
            );

            &block.prog_lines
        };

        for line in prog_lines {
            writeln!(prg, "{}", line)?;
        }
        writeln!(prg, "printf \"${{}_{}}\"", vars.output_sep_nonce, i)?;
        writeln!(prg, "")?;
    }
    prg.flush()?;

    let proc = process::Command::new("bash")
        .arg(&program_path)
        .env(vars.source_file_path, path::absolute(file_path)?)
        .envs(output_sep_nonces.iter().enumerate().map(|(i, nonce_val)| {
            let nonce_var = format!("{}_{}", vars.output_sep_nonce, i);
            (nonce_var, nonce_val)
        }))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let output_path = temp_dir_path.join("output").with_extension(file_ext);
    let mut output_file = File::create_buffered(output_path)?;
    let mut output_reader = BufReader::new(proc.stdout.unwrap());

    // prepend one "None" block at the beginning for the prologue (terminated by nonce 0)
    let blocks_and_nonces = zip(
        chain(once(None), blocks.iter().map(Some)),
        output_sep_nonces,
    );

    for (block_opt, output_end_nonce) in blocks_and_nonces {
        if let Some(block) = block_opt {
            // write portion of file before block starts
            let pfx_start = block.markers[MarkerKind::OutputEnd as usize].span.end();
            let pfx_end = block.markers[MarkerKind::ProgramEnd as usize].span.end();
            write!(output_file, "{}", &ctx.content[pfx_start..pfx_end]);
        }

        let block_output: io::Result<String> = {
            let mut buf = String::new();
            loop {
                let n = output_reader.read_line(&mut buf)?;
                if n == 0 {
                    return io::Error::other("error");
                }

                if let Some(block_output) = buf.strip_suffix(&output_end_nonce) {
                    todo!()
                }
            }
        };
    }

    // for block_pairs in capped_blocks {
    //     match block_pairs {
    //         [None, Some(block)] => {}
    //     }
    // }

    // for i in 0..=blocks.len() {
    //     if i < blocks.len() {
    //         let block = &blocks[i];
    //         let [prog_start, prog_end, outp_end] = &block.markers;
    //         write!(output_file, "{}", &ctx.content[..prog_end.span.end()])?;
    //     }
    // }
    Ok(())
}

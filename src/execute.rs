use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
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
    let exec_dir = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
    let exec_dir_path = exec_dir.path();

    // create nonces to figure out where output of one block ends and another begins
    let output_sep_nonces: Vec<_> = {
        // one for each block plus one for the prologue
        let n = blocks.len() + 1;
        iter::repeat_with(|| Uuid::new_v4().to_string())
            .take(n)
            .collect()
    };

    let vars = &ctx.config.env_var_names;
    let program_path = exec_dir_path.join("program.sh");
    let mut prg = File::create_buffered(&program_path)?;

    writeln!(prg, "#!/usr/bin/env bash")?;

    for i in 0..blocks.len() + 1 {
        let lines = if i == 0 {
            writeln!(prg, "echo 'executing prologue for {}...' >&2", ctx.filename);
            &ctx.config.prologue
        } else {
            let block = &blocks[i - 1];

            let path = exec_dir_path
                .join("output_prev")
                .with_added_extension((i + 1).to_string())
                .with_added_extension(&file_ext);
            fs::write(&path, &ctx.content)?;
            writeln!(prg, "export {}={}", vars.output_prev, path.display())?;

            let prog_start = block.markers[0];
            writeln!(prg, "export {}={}", vars.prog_start_line, prog_start.line)?;
            writeln!(prg, "export {}={}", vars.prog_start_col, prog_start.col)?;
            writeln!(
                prg,
                "export {}={}",
                vars.prog_start_offset,
                prog_start.span.start()
            )?;
            writeln!(
                prg,
                "echo 'executing block at {}:{}:{}...' >&2",
                ctx.filename, prog_start.line, prog_start.col
            );

            &block.prog_lines
        };

        for line in lines {
            writeln!(prg, "{}", line)?;
        }
        writeln!(prg, r"echo -e '\n{}'", output_sep_nonces[i])?;
        writeln!(prg, "")?;
    }
    prg.flush()?;

    let proc = process::Command::new("bash")
        .arg(&program_path)
        .env(vars.source_file_path, path::absolute(file_path)?)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let proc_out_lines = BufReader::new(proc.stdout.unwrap()).lines();

    let output_path = exec_dir_path.join("output").with_extension(file_ext);
    let mut output_file = File::create_buffered(output_path)?;

    let mut prev_offset = 0;
    let blocks_and_nonces = zip(
        chain(once(None), blocks.iter().map(Some)),
        output_sep_nonces,
    );

    for (block_opt, output_end_nonce) in blocks_and_nonces {

      let output = proc_out_lines

      if let Some(block) = block_opt {

        // write portion of file until output block start
        let pfx_start = match block_prev {
            Some(block) => block.markers[MarkerKind::OutputEnd as usize].span.end(),
            None => 0,
        };
        let pfx_end = match block_next {
            Some(block) => block.markers[MarkerKind::ProgramEnd as usize].span.end(),
            None => ctx.content.len(),
        };
        write!(output_file, "{}", &ctx.content[pfx_start..pfx_end]);

        let block_output = if let Some(block) = block_next
            && let [_, prog_end, outp_end] = &block.markers
            && prog_end.line != outp_end.line
        {};
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

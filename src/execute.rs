use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::iter::{self};
use std::path::{self, PathBuf};
use std::process::{self, Output, Stdio};

use anyhow::Result;
use regex::bytes::Regex;
use uuid::Uuid;

use crate::parse::ParsedFile;

pub fn execute(ParsedFile { ctx, blocks }: &ParsedFile) -> Result<()> {
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
    let mut prg = BufWriter::new(File::open(&program_path)?);

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

            let (line, col, offset) = block.prog_start_loc;
            writeln!(prg, "export {}={}", vars.prog_start_line, line)?;
            writeln!(prg, "export {}={}", vars.prog_start_col, col)?;
            writeln!(prg, "export {}={}", vars.prog_start_offset, offset)?;
            writeln!(
                prg,
                "echo 'executing block at {}:{}:{}...' >&2",
                ctx.filename, line, col
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

    let output_file = exec_dir_path.join("output").with_extension(file_ext);

    for ()
}

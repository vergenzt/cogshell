use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::iter::{self};
use std::path::{self, PathBuf};
use std::process;

use anyhow::Result;
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
        iter::repeat_with(Uuid::new_v4).take(n).collect()
    };

    let vars = &ctx.config.env_var_names;
    let program_path = exec_dir_path.join("program.sh");
    let mut prg = BufWriter::new(File::open(&program_path)?);

    writeln!(prg, "#!/usr/bin/env bash")?;
    for line in &ctx.config.prologue {
        writeln!(prg, "{}", line)?;
    }
    writeln!(prg, "echo '{}'", output_sep_nonces[0])?;
    writeln!(prg, "")?;

    for (i, block) in blocks.iter().enumerate() {
        for (varname, basename, content) in [
            (vars.output_prev, "output_prev", block.output_prev),
            (vars.output_next, "output_next", ""),
        ] {
            let path = exec_dir_path
                .join(basename)
                .with_added_extension((i + 1).to_string())
                .with_added_extension(&file_ext);
            fs::write(&path, content)?;
            writeln!(prg, "export {}={}", varname, path.display())?;
        }
        let (line, col, offset) = block.prog_start_loc;
        writeln!(prg, "export {}={}", vars.prog_start_line, line)?;
        writeln!(prg, "export {}={}", vars.prog_start_col, col)?;
        writeln!(prg, "export {}={}", vars.prog_start_offset, offset)?;
        for line in &block.prog_lines {
            writeln!(prg, "{}", line)?;
        }
        writeln!(prg, "echo '{}'", output_sep_nonces[i + 1])?;
        writeln!(prg, "")?;
    }

    let cmd = process::Command::new("bash")
        .arg(&program_path)
        .env(vars.source_file_path, path::absolute(file_path)?);
}

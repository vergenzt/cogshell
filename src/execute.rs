use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::iter::{self};
use std::os::unix::fs::PermissionsExt;
use std::path::{self, PathBuf};
use std::process::Stdio;
use std::str::pattern::Pattern;
use std::{array, process};

use anyhow::{Context, Result};
use fastrand::alphanumeric as rand_alnum;
use uuid::Uuid;

use crate::config::Config;
use crate::parse::{ParsedBlock, ParsedFile};

const OUTPUT_SEP_NONCE_LEN: usize = 40;

pub fn execute(file: &ParsedFile) -> Result<()> {
    let file_path = PathBuf::from(file.filename);
    let file_ext: OsString = match file_path.extension() {
        Some(ext) => ext.into(),
        None => "".into(),
    };
    let exec_dir = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
    let exec_dir_path = exec_dir.path();

    let content_line_starts: Vec<_> = file.content.match_indices('\n').map(|(i, _)| i).collect();

    // create nonces to figure out where output of one block ends and another begins
    let output_sep_nonces: Vec<_> = {
        // one for each block plus one for the prologue
        let n = file.blocks.len() + 1;
        iter::repeat_with(Uuid::new_v4).take(n).collect()
    };

    let vars = &file.config.env_var_names;
    let program_path = exec_dir_path.join("program.sh");

    let output_sep_nonce_vars = output_sep_nonces
        .iter()
        .enumerate()
        .map(|(i, nonce)| (format!("OUTPUT_NONCE_{}", i), nonce.to_string()));

    let output_path_vars = file.blocks.iter().enumerate().flat_map(|(i, block)| {
        [
            ("output_prev", vars.output_prev, block.output_prev),
            ("output_next", vars.output_next, ""),
        ]
        .map(|(basename, varname, content)| {
            let path = exec_dir_path
                .join(basename)
                .with_added_extension((i + 1).to_string())
                .with_added_extension(&file_ext);
            fs::write(&path, content);
            (varname, path)
        })
    });

    let cmd = process::Command::new("bash")
        .arg(program_path)
        .env(vars.source_file_path, path::absolute(file_path)?)
        .envs(output_sep_nonce_vars)
        .envs(output_path_vars);

    let mut prg: Vec<&str> = vec![];

    prg.push("#!/usr/bin/env bash");
    prg.extend(&file.config.prologue);

    let output_sep_nonce0 = format!("echo '{}'", &output_sep_nonces[0]);
    prg.push(&output_sep_nonce0);

    for (block_idx, block) in file.blocks.iter().enumerate() {
        let out_prev = exec_dir_path
            .join("output_prev")
            .with_added_extension(format!("{}", block_idx + 1))
            .with_added_extension(&file_ext);
        fs::write(out_prev, block.output_prev);
        prg.push(&format!(
            "export {}={}",
            vars.output_prev,
            out_prev.display()
        ));
        prg.extend(block.prog_lines);
    }

    todo!()
}

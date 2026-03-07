use std::io::{self, BufRead, Write};
use std::ops::Range;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::fs;

use anyhow::{bail, Context, Result};
use md5::{Digest, Md5};

mod config;
mod parse;
mod utils;
use config::{Config, LINE_SEPS};
use parse::ParsedBlock;


struct ExecutedBlock {
    parsed: ParsedBlock,
}

impl ExecutedBlock {
    fn execute(&self) -> Result<()> {
        if self.config.output_checksum {
            // Note: Python's `if not hash_match` was always False (re.Match objects are always
            // truthy); the intent appears to be checking whether the hash capture group matched.
            match &self.output_prev_hash_match {
                None => eprintln!(
                    "Warning: {}({}): output_checksum=true but no output checksum detected",
                    self.filename.display(),
                    self.output_end_line,
                ),
                Some((line_no, stored_hash)) if stored_hash != &self.output_prev_hash => {
                    bail!(
                        "Output checksum does not match! {}({})",
                        self.filename.display(),
                        line_no,
                    );
                }
                _ => {}
            }
        }

        let exec_dir = tempfile::Builder::new()
            .prefix("cogshell-")
            .tempdir()?;
        let exec_dir_path = exec_dir.path();

        let program_file = exec_dir_path.join("program");
        let output_prev_path = exec_dir_path.join("output_prev");
        let output_next_path = exec_dir_path.join("output_next");

        fs::write(&program_file, &self.program)?;
        let mut perms = fs::metadata(&program_file)?.permissions();
        perms.set_mode(0o500); // S_IEXEC | S_IREAD
        fs::set_permissions(&program_file, perms)?;
        fs::write(&output_prev_path, &self.output_prev)?;
        fs::File::create(&output_next_path)?;

        let names = &self.config.env_names;
        let prefix = &self.config.envvar_prefix;
        let env_vars = [
            (&names.file,        fs::canonicalize(&self.filename)?.to_string_lossy().into_owned()),
            (&names.program,     program_file.to_string_lossy().into_owned()),
            (&names.output_prev, output_prev_path.to_string_lossy().into_owned()),
            (&names.output_next, output_next_path.to_string_lossy().into_owned()),
            (&names.exec_dir,    exec_dir_path.to_string_lossy().into_owned()),
        ]
        .map(|(name, val)| (format!("{prefix}{name}"), val));

        let stdin = if self.config.prev_output_on_stdin {
            Stdio::from(fs::File::open(&output_prev_path)?)
        } else {
            Stdio::null()
        };
        let stdout = if self.config.next_output_on_stdout {
            Stdio::from(fs::File::create(&output_next_path)?)
        } else {
            Stdio::inherit()
        };

        std::process::Command::new("sh")
            .arg("-c")
            .arg(std::str::from_utf8(&self.program).context("program is valid utf8")?)
            .current_dir(exec_dir_path)
            .envs(env_vars)
            .stdin(stdin)
            .stdout(stdout)
            .status()?
            .exit_ok()?;

        let mut hasher = Md5::new();
        let mut last_output_line: Vec<u8> = vec![];

        let mut out_file = fs::File::create(&self.filename)?;
        out_file.write_all(&self.file_prefix)?;

        for line_sep in LINE_SEPS {
            if self.output_prev_raw.starts_with(line_sep) {
                out_file.write_all(b"\n")?;
                break;
            }
        }

        let mut reader = io::BufReader::new(fs::File::open(&output_next_path)?);
        let mut raw_line: Vec<u8> = Vec::new();
        loop {
            raw_line.clear();
            if reader.read_until(b'\n', &mut raw_line)? == 0 {
                break;
            }
            let line_sep = LINE_SEPS
                .iter()
                .find(|sep| raw_line.ends_with(*sep))
                .copied()
                .unwrap_or(b"");
            let output_line_raw = &raw_line[..raw_line.len() - line_sep.len()];
            let output_line: Vec<u8> = [
                self.output_whitespace_pfx.as_slice(),
                output_line_raw,
                &self.config.output_line_suffix,
                line_sep,
            ]
            .concat();
            out_file.write_all(&output_line)?;
            hasher.update(&output_line);
            last_output_line = output_line;
        }

        for line_sep in LINE_SEPS {
            if self.output_prev_raw.ends_with(line_sep) && !last_output_line.ends_with(line_sep) {
                out_file.write_all(line_sep)?;
                out_file.write_all(&self.output_whitespace_pfx)?;
            }
        }

        out_file.write_all(&self.config.markers.output_end)?;
        if self.config.output_checksum {
            write!(out_file, " ({:x})", hasher.finalize())?;
        }
        out_file.write_all(&self.file_suffix)?;

        Ok(())
    }
}

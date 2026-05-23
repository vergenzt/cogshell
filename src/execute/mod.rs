pub mod terminators;

use std::fs;
use std::io::{self, BufRead as _, Write as _};
use std::iter;
use std::path;
use std::process::{self, Stdio};

use tempfile::TempDir;

use crate::deref_field;

use super::args::FileArg;
use super::parse::ParsedFile;
use terminators::OutputTerminator;

pub struct FileExecutor<'a, 'i> {
    pub file: &'i ParsedFile<'a, 'i>,
    cmd: process::Command,
    output_terminators: Vec<OutputTerminator>,
    _temp_dir: TempDir,
}

deref_field! { impl<'a, 'i> *FileExecutor<'a, 'i> = .file: ParsedFile<'a, 'i> }

impl<'a, 'i> FileExecutor<'a, 'i> {
    pub fn initialize(file: &'i ParsedFile<'a, 'i>) -> io::Result<FileExecutor<'a, 'i>> {
        let source = file.input.source.to_string();
        let source_ext = source.rsplit_terminator('.').next().unwrap_or("");
        let temp_dir = tempfile::Builder::new().prefix("cogshell-").tempdir()?;
        let temp_path = temp_dir.path();

        let mut env: Vec<(String, String)> = vec![];

        macro_rules! var {
            ($namefmt:literal => $val:expr) => {
                env.push((format!("COGSH_{}", $namefmt), $val));
            };
            ($namefmt:literal [$i:expr] => $val:expr) => {
                env.push((format!("COGSH_{}_{}", $namefmt, $i), $val));
            };
        }
        macro_rules! path {
            ($fname:expr, $content:expr) => {{
                let path = temp_path.join($fname);
                fs::write(&path, $content)?;
                path.to_str().unwrap().to_owned()
            }};
        }

        let output_terminators: Vec<_> = iter::repeat_with(OutputTerminator::new)
            .take(file.blocks.len() + 1)
            .collect();

        var!("TEMP_DIR" => temp_path.to_str().unwrap().to_owned());
        var!("SOURCE" => source.clone());
        var!("NUM_BLOCKS" => file.blocks.len().to_string());
        var!("PROLOGUE" => path!("prologue.sh", file.input.args.prologue.join("\n")));

        for (i0, block) in file.blocks.iter().enumerate() {
            let i1 = i0 + 1;

            let span = block.markers.prog_start.span;
            var!("BLOCK_LINE" [i1] => span.start.line.to_string());
            var!("BLOCK_COL" [i1] => span.start.col.to_string());
            var!("BLOCK_OFFSET" [i1] => span.start.offset.to_string());

            var!("BLOCK_PROG" [i1] => path!(format!("block_{i1}.sh"), block.prog_lines.join("\n")));
            var!("BLOCK_OUTPUT_LINE_PFX" [i1] => block.prog_whitespace_pfx.to_owned());
            var!("BLOCK_OUTPUT_PREV" [i1] => path!(format!("output_prev_{i1}{source_ext}"), block.output_prev));
        }

        let mut cmd = process::Command::new("bash");
        cmd.args(["-c", include_str!("executor.sh")]);
        cmd.arg(source.clone());
        for term in &output_terminators {
            cmd.arg(&**term);
        }
        cmd.envs(env);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::inherit());

        if let FileArg::OnDisk(input_path) = &file.input.source.arg {
            let input_path_abs = path::absolute(input_path)?;
            if let Some(parent) = input_path_abs.parent() {
                cmd.current_dir(parent);
            }
        }

        Ok(FileExecutor {
            file,
            cmd,
            output_terminators,
            _temp_dir: temp_dir,
        })
    }

    pub fn execute(&mut self, output: &mut dyn io::Write) -> io::Result<()> {
        let mut proc = self.cmd.spawn()?;
        let stdout = proc.stdout.take().expect("child stdout missing");
        let mut proc_reader = io::BufReader::new(stdout);

        let suffix = self.file.input.args.output_line_suffix.clone();
        let content = &self.file.input.content;

        // 1. Drain prologue output to stderr (decorated).
        {
            let source_label = self.file.input.source.to_string();
            let prefix = format!("[PROLOGUE {source_label}] ");
            let term: &str = &self.output_terminators[0];
            let mut prev: Option<String> = None;
            let stderr = io::stderr();
            let mut err = stderr.lock();
            loop {
                let mut line = String::new();
                if proc_reader.read_line(&mut line)? == 0 {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
                if line.strip_suffix('\n') == Some(term) {
                    if let Some(p) = prev.take()
                        && p != "\n"
                    {
                        err.write_all(prefix.as_bytes())?;
                        err.write_all(p.as_bytes())?;
                    }
                    break;
                }
                if let Some(p) = prev.take() {
                    err.write_all(prefix.as_bytes())?;
                    err.write_all(p.as_bytes())?;
                }
                prev = Some(line);
            }
        }

        // 2. Walk content + blocks.
        let mut cursor: usize = 0;
        let blocks = &self.file.blocks;
        for (i, block) in blocks.iter().enumerate() {
            let prog_marker_end = *block.markers.prog_end.span.end;
            let prog_line_end = content[prog_marker_end..]
                .find('\n')
                .map(|n| prog_marker_end + n + 1)
                .unwrap_or(content.len());

            let outp_marker_start = *block.markers.outp_end.span.start;
            let outp_line_start = content[..outp_marker_start]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);

            output.write_all(content[cursor..prog_line_end].as_bytes())?;

            let prefix = block.prog_whitespace_pfx;
            let term: &str = &self.output_terminators[i + 1];
            let mut prev: Option<String> = None;
            loop {
                let mut line = String::new();
                if proc_reader.read_line(&mut line)? == 0 {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
                if line.strip_suffix('\n') == Some(term) {
                    if let Some(p) = prev.take()
                        && p != "\n"
                    {
                        write_block_line(output, prefix, &suffix, &p)?;
                    }
                    break;
                }
                if let Some(p) = prev.take() {
                    write_block_line(output, prefix, &suffix, &p)?;
                }
                prev = Some(line);
            }

            cursor = outp_line_start;
        }

        // 3. Write rest of file.
        output.write_all(content[cursor..].as_bytes())?;
        output.flush()?;

        let status = proc.wait()?;
        if !status.success() {
            return Err(io::Error::other(format!(
                "cogsh subprocess exited with {status}"
            )));
        }
        Ok(())
    }
}

fn write_block_line(
    out: &mut dyn io::Write,
    prefix: &str,
    suffix: &str,
    line: &str,
) -> io::Result<()> {
    let trimmed = line.strip_suffix('\n').unwrap_or(line);
    out.write_all(prefix.as_bytes())?;
    out.write_all(trimmed.as_bytes())?;
    out.write_all(suffix.as_bytes())?;
    out.write_all(b"\n")?;
    Ok(())
}

use std::fs::{self};
use std::io::{self, BufRead as _, BufReader, Write};
use std::iter::{self};
use std::path::PathBuf;
use std::process::{self, Stdio};

use uuid::Uuid;

use crate::args::io::FileOrStream;
use crate::parse::{BlockMarkers, File, MarkerInst, Span};

pub fn execute(file: &File) -> io::Result<()> {
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

    var!("TEMP_DIR" => temp_dir.display());
    var!("SOURCE" => source_name);
    var!("NUM_BLOCKS" => blocks.len());
    if ctx.config.prologue.len() > 0 {
        var!("PROLOGUE" => path!("prologue.sh", ctx.config.prologue.join("\n")));
    }
    var!("PROLOGUE_SUFFIX" => path!(format!("prologue_suffix{source_ext}"), &ctx.content[..blocks[0].start()]));

    for (i0, block) in blocks.iter().enumerate() {
        let i1 = i0 + 1; // 1-based indexing

        let span = block.markers.prog_beg.span;
        var!("BLOCK_LINE" i1 => span.line);
        var!("BLOCK_COL" i1 => span.col);
        var!("BLOCK_OFFSET" i1 => span.start);

        var!("BLOCK_PROG" i1 => path!(format!("block_{i1}.sh"), block.prog_lines.join("\n")));
        var!("BLOCK_OUTPUT_LINE_PFX" i1 => block.prog_whitespace_pfx);
        var!("BLOCK_OUTPUT_PREV" i1 => path!(format!("output_prev_{i1}{source_ext}"), block.output_prev));

        let block_suffix = match blocks.get(i1) {
            Some(next_block) => &ctx.content[block.end()..next_block.start()],
            None => &ctx.content[block.end()..],
        };
        var!("BLOCK_SUFFIX" i1 => path!(format!("block_suffix_{i1}{source_ext}"), block_suffix));
    }

    let output_path_tmp = temp_dir.join(format!("output{source_ext}"));

    let mut cmd = process::Command::new("bash");
    cmd.args(["-c", include_str!("program.sh")]);
    cmd.arg(&source_name); // make $COGSH_SOURCE also available as $0
    cmd.envs(env);
    cmd.stdin(Stdio::null());
    cmd.stderr(Stdio::inherit());

    let run = |mut cmd: process::Command| -> io::Result<()> {
        cmd.spawn()?.wait()?.exit_ok().map_err(io::Error::other)
    };

    if let FileOrStream::File(source_path, _) = &ctx.input {
        let output_file_tmp = fs::File::create(&output_path_tmp)?;
        cmd.stdout(output_file_tmp);

        run(cmd)?;

        fs::rename(output_path_tmp, source_path)?
    } else {
        run(cmd)?;
    }

    Ok(())
}

#![feature(trim_prefix_suffix)]

use std::io;
use std::process::ExitCode;

mod args;
mod deref_util;
mod execute;
mod parse;

pub(crate) use deref_util::*;

use args::{Args, File, FileArg, Read, Write};
use execute::FileExecutor;
use parse::{ParseInput, ParsedFile};

pub fn main() -> ExitCode {
    let args = Args::to_options().run();
    match run_cogshell(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("cogshell: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_cogshell(args: &Args) -> io::Result<()> {
    match &args.output {
        Some(out_arg) => {
            let out_file: File<Write> = out_arg.into();
            let mut output = out_file.open()?;
            for src_arg in &args.files {
                read_and_process(args, src_arg, &mut *output)?;
            }
        }
        None => {
            for src_arg in &args.files {
                let out_file: File<Write> = src_arg.into();
                let mut output = out_file.open()?;
                read_and_process(args, src_arg, &mut *output)?;
            }
        }
    }
    Ok(())
}

fn read_and_process(args: &Args, src_arg: &FileArg, output: &mut dyn io::Write) -> io::Result<()> {
    let mut src_file: File<Read> = src_arg.into();
    let input = ParseInput::from(args, &mut src_file)?;
    process(&input, output)
}

fn process(input: &ParseInput<'_>, output: &mut dyn io::Write) -> io::Result<()> {
    let parsed = ParsedFile::from(input)?;
    let mut exec = FileExecutor::initialize(&parsed)?;
    exec.execute(output)
}

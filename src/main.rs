#![feature(trim_prefix_suffix)]

use std::io;
use std::process::ExitCode;

mod args;
mod deref_util;
mod execute;
mod parse;

pub(crate) use deref_util::*;

use args::{Args, File, FileArg, Read};
use execute::FileExecutor;
use parse::{ParseInput, ParsedFile};

pub fn main() -> ExitCode {
    let args = Args::to_options().run();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("cogsh: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> io::Result<()> {
    match &args.output {
        Some(out_arg) => {
            let out_file: File<args::Write> = out_arg.into();
            let mut writer = out_file.open()?;
            for src_arg in &args.files {
                process_one(args, src_arg, &mut *writer)?;
            }
        }
        None => {
            for src_arg in &args.files {
                let out_file: File<args::Write> = src_arg.into();
                let mut writer = out_file.open()?;
                process_one(args, src_arg, &mut *writer)?;
            }
        }
    }
    Ok(())
}

fn process_one(args: &Args, src_arg: &FileArg, writer: &mut dyn io::Write) -> io::Result<()> {
    let mut src_file: File<Read> = src_arg.into();
    let input = ParseInput::from(args, &mut src_file)?;
    // input owns its content; parsed file borrows from input
    process_with_input(&input, writer)
}

fn process_with_input(input: &ParseInput<'_>, writer: &mut dyn io::Write) -> io::Result<()> {
    let parsed = ParsedFile::from(input)?;
    let mut exec = FileExecutor::initialize(&parsed)?;
    exec.execute(writer)
}

#![feature(trim_prefix_suffix)]
#![feature(file_buffered)]
#![feature(iterator_try_collect)]

use std::{io::stderr, thread};

mod args;
mod errors;
mod execute;
mod parse;
mod deref_util;

pub(crate) use deref_util::*;

pub fn main() {
    let args = args::Args::to_options().run();
    let threads = args.files.map(|f| {
      thread::spawn(|| {

      })
    })
    match args.files {
        args::SourceAndDestArgs::SourcesInPlace(ref pipes) => {
            for pipe in pipes {
                let fctx = parse::ParsedFile::new(pipe, &args).unwrap();
                let file = parse::ParsedFile::from(&fctx).unwrap();
                let mut exec = execute::FileExecutor::initialize(&file).unwrap();
                exec.execute(pipe).unwrap();
            }
        }
        args::SourceAndDestArgs::SingleSourceAndDest(ref src, ref dst) => {
            let fctx = parse::ParsedFile::new(src, &args).unwrap();
            let file = parse::ParsedFile::from(&fctx).unwrap();
            let mut exec = execute::FileExecutor::initialize(&file).unwrap();
            exec.execute(dst).unwrap();
        }
    }
}

#![feature(trim_prefix_suffix)]
#![feature(file_buffered)]

mod args;
mod execute;
mod parse;

pub fn main() {
    let args = args::Args::to_options().run();
    match args.source_and_dest {
        args::SourceAndDestArgs::SourcesInPlace(ref pipes) => {
            for pipe in pipes {
                let fctx = parse::FileContext::new(pipe, &args).unwrap();
                let file = parse::File::from(&fctx).unwrap();
                let mut exec = execute::FileExecutor::new(&file).unwrap();
                exec.execute(pipe).unwrap();
            }
        }
        args::SourceAndDestArgs::SingleSourceAndDest(ref src, ref dst) => {
            let fctx = parse::FileContext::new(src, &args).unwrap();
            let file = parse::File::from(&fctx).unwrap();
            let mut exec = execute::FileExecutor::new(&file).unwrap();
            exec.execute(dst).unwrap();
        }
    }
}

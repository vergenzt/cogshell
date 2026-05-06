use core::error::Source;
use std::{ffi::OsString, ops::Index, path::PathBuf};

use bpaf::{Bpaf as BpafArgParser, Parser, construct, long, positional};

use crate::config::MarkerConfig;

/// Strategy describing *how* to write to an output destination
pub enum DestWriteStrategy {
    /// Buffer output to a temporary file, then only upon successful completion either
    /// rename it (for file destinations) or stream it all (for stdout).
    Atomic,
    /// Write directly to the file or stream as output comes available. May produce partial
    /// output if an error occurs. Truncates file destinations on start.
    Immediate,
}

/// A "path" to read or write from (where `-` means stdin or stdout)
pub enum PathArg {
    /// Stdin or stdout
    Stream,
    File(PathBuf),
}

impl PathArg {
    pub fn parse(path: PathBuf) -> Self {
        match path {
            [b'-'] => Self::Stream,
            _ => Self::File(path),
        }
    }
}

pub enum SourceAndDestArgs {
    /// Process any number of files in-place
    SourcesInPlace(Vec<PathArg>),
    /// Process exactly one input (stdin or file) to one output (stdout or file)
    SourceAndDest(PathArg, PathArg),
}

impl SourceAndDestArgs {
    fn parser() -> impl Parser<Self> {
        let source = positional::<PathBuf>("SOURCE").map(PathArg::parse);
        let dest = long("output")
            .short('o')
            .argument::<PathBuf>("DEST")
            .map(PathArg::parse);
        let source_to_dest = construct!(SourceAndDestArgs::SourceAndDest(source, dest));

        let sources = source.some("Must specify at least one SOURCE!");
        let sources_in_place = construct!(SourceAndDestArgs::SourcesInPlace(sources));

        construct!([source_to_dest, sources_in_place])
    }
}

pub struct Args {
    source_and_dest: SourceAndDestArgs,
    prologue: Vec<&OsStr>,
}

pub struct Args {
    #[bpaf(positional("FILE"))]
    files: Vec<FileOrStream<In>>,

    #[bpaf(long("output"), short('o'), argument("OUTNAME"))]
    /// Write the output to OUTNAME instead of inline
    output: Option<FileOrStream>,

    #[bpaf(long("prologue"), short('p'), argument::<OsString>("PROLOGUE"), parse(|s| Ok(s.into_encoded_bytes())), many)]
    /// Prepend CogShell code in a file with PROLOGUE. Executed once per file before the first CogShell block.
    prologue: Vec<Vec<u8>>,

    #[bpaf(
        long("markers"),
        argument::<MarkerConfig>("START END END-OUTPUT"),
        fallback(MarkerConfig::from_str("[[[cogsh ]]] [[[end]]]").unwrap()),
        display_fallback,
    )]
    /// The patterns surrounding cog inline instructions. Should include three
    /// values separated by spaces, the start, end, and end-output markers.
    markers: MarkerConfig,
}

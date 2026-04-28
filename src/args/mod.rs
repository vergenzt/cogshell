pub mod io;

use std::ffi::OsString;

use bpaf::Bpaf as ArgParser;

use io::{FileOrStream, In, Out};

use crate::config::MarkerConfig;

#[derive(ArgParser, Debug, Clone)]
#[bpaf(options)]
pub struct Args {
    #[bpaf(positional("FILE"))]
    files: Vec<FileOrStream<In>>,

    #[bpaf(long("output"), short('o'), argument("OUTNAME"))]
    /// Write the output to OUTNAME instead of inline
    output: Option<FileOrStream<Out>>,

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

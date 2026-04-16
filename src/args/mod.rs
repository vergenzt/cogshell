mod io;

use bpaf::Bpaf as ArgParser;

use io::{FileOrStream, In, Out};
use std::str::FromStr;

use crate::config::MarkerConfig;

#[derive(ArgParser, Debug, Clone)]
#[bpaf(options)]
pub struct Args {
    #[bpaf(positional("FILE"))]
    files: Vec<FileOrStream<In>>,

    #[bpaf(short('c'), fallback(true), display_fallback)]
    /// Checksum the output to protect it against accidental change
    checksum_output: bool,

    #[bpaf(short('C'), fallback(true), display_fallback)]
    /// Validate existing cheksums found before re-executing
    checksum_validate: bool,

    #[bpaf(long("output"), short('o'), argument("OUTNAME"))]
    /// Write the output to OUTNAME instead of inline
    output: Option<FileOrStream<Out>>,

    #[bpaf(long("prologue"), short('p'))]
    /// Prepend CogShell code in a file with PROLOGUE. Executed once per file before the first CogShell block.
    prologue: Vec<String>,

    #[bpaf(
        long("markers"),
        argument("START END END-OUTPUT"),
        fallback(MarkerConfig::from_str("[[[cogsh ]]] [[[end]]]").ok()),
        display_fallback
    )]
    /// The patterns surrounding cog inline instructions. Should include three
    /// values separated by spaces, the start, end, and end-output markers.
    markers: Option<MarkerConfig>,
}

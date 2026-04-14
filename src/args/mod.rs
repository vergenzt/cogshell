mod path_or_stdin;

use std::path::PathBuf;

use bpaf::*;

use crate::args::path_or_stdin::PathOrStdin;

fn parse_markers(s: &str) -> Result<(&str, &str, &str), String> {
    match s.split_ascii_whitespace().collect::<Vec<_>>().as_ {
        [a, b, c] => Ok((a, b, c)),
        _ => Err(),
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Args {
    #[bpaf(positional)]
    /// File(s) to process with CogShell, or "-" for stdin
    files: Vec<PathOrStdin>,

    #[bpaf(fallback(true), display_fallback)]
    /// Checksum the output to protect it against accidental change
    checksum: bool,

    #[bpaf(argument("OUTNAME"))]
    /// Write the output to OUTNAME instead of inline
    output: Option<PathBuf>,

    #[bpaf(short('p'))]
    /// Prepend CogShell code in a file with PROLOGUE. Executed once per file before the first CogShell block.
    prologue: Vec<String>,

    #[bpaf(
      parse(
        |s| s.split_ascii_whitespace().collect::<Vec<_>>().as_array().ok_or(format!(
          "Marker value {s:?} does not contain three whitespace-separated values!"
        ))
      ),
      fallback(["[[[cogsh", "]]]", "[[[end]]]"])
    )]
    /// The patterns surrounding cog inline instructions. Should include three
    /// values separated by spaces, the start, end, and end-output markers.
    /// Defaults to '[[[cogsh ]]] [[[end]]]'.
    markers: [String; 3],
    // markers: Markers = Markers("[[[cog", "]]]", "[[[end]]]")
    // _parser.add_argument(
    //     "--markers",
    //     metavar="'START END END-OUTPUT'",
    //     type=Markers.from_arg,
    //     help=dedent("""
    //
    //
    //
    //     """),
    // )
}

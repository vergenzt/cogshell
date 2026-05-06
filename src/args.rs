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
    StdStream,
    File(PathBuf),
}

impl PathArg {
    /// Interpret exactly `-` as std{in/out}, anything else as itself.
    pub fn parse(path: PathBuf) -> Self {
        match path {
            [b'-'] => Self::StdStream,
            _ => Self::File(path),
        }
    }
}

/// Enum to restrict combinations of source/dest args. (Output destination can only be
/// specified if there is exactly one source arg.)
pub enum SourceAndDestArgs {
    /// Process any number of files in-place
    SourcesInPlace(Vec<PathArg>),
    /// Process exactly one input (stdin or file) to one output (stdout or file)
    SourceAndDest(PathArg, PathArg),
}

impl SourceAndDestArgs {
    fn source() -> impl Parser<PathArg> {
        positional::<PathBuf>("SOURCE").map(PathArg::parse)
    }

    fn dest() -> impl Parser<PathArg> {
        long("output")
            .short('o')
            .argument::<PathBuf>("DEST")
            .map(PathArg::parse)
    }

    fn source_and_dest() -> impl Parser<SourceAndDestArgs> {
        construct!(SourceAndDestArgs::SourceAndDest(source(), dest()))
    }

    fn sources() -> impl Parser<Vec<PathArg>> {
        source().some("Must specify at least one SOURCE!")
    }

    fn sources_in_place() -> impl Parser<SourceAndDestArgs> {
        construct!(SourceAndDestArgs::SourcesInPlace(sources()))
    }

    fn parser() -> impl Parser<Self> {
        construct!([source_and_dest(), sources_in_place()])
    }
}

pub struct MarkerConfig([OsString; 3]);

impl MarkerConfig {
    fn parse(arg: OsString) -> Result<Self, String> {
        let parts: Vec<_> = arg.into_vec().split(|c| c.is_ascii_whitespace()).collect();
        match parts[..] {
            [a, b, c] => Ok(Self([a, b, c].map(OsString::from))),
            _ => {
                let disp = arg.display();
                Err(format!(
                    "Marker config `{disp}` does not consist of three whitespace-separated words!"
                ))
            }
        }
    }

    /// The patterns surrounding cog inline instructions. Should include three
    /// values separated by spaces, the start, end, and end-output markers.
    fn parser() -> impl Parser<Self> {
        long("markers")
            .argument::<OsString>("START END END-OUTPUT")
            .fallback(OsString::from("[[[cogsh ]]] [[[end]]]"))
            .parse(parse)
            .display_fallback()
    }
}

impl Deref for MarkerConfig {
    type Target = [OsString; 3];
    fn deref(self: &MarkerConfig) -> &Self::Target {
        &self.0
    }
}

pub struct Args {
    source_and_dest: SourceAndDestArgs,
    prologue: Vec<OsString>,
    output_line_suffix: Option<OsString>,
    markers: MarkerConfig,
}

impl Args {
    /// Statements to execute before running CogShell scripts. Executed once per file before the first block.
    fn prologue() -> impl Parser<Vec<OsString>> {
        long("prologue").argument("LINE").many()
    }

    fn output_line_suffix() -> impl Parser<OsString> {
        long("suffix").short('s').argument("SUFFIX")
    }

    pub fn parser() -> impl Parser<Self> {
        let source_and_dest = SourceAndDestArgs::parser();
        let markers = MarkerConfig::parser();
        construct!(Self {
            source_and_dest,
            prologue(),
            output_line_suffix(),
            markers,
        })
    }
}

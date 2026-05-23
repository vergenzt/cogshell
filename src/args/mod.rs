pub(crate) mod files;
pub(crate) mod marker_defs;

use std::str::FromStr as _;

pub use files::{File, FileArg, Read, Write};
pub use marker_defs::MarkerDefs;

use bpaf::{OptionParser, Parser, construct, long, positional};

#[derive(Debug)]
pub struct Args {
    pub files: Vec<FileArg>,
    pub output: Option<FileArg>,
    pub prologue: Vec<String>,
    pub output_line_suffix: String,
    pub markers: MarkerDefs,
}

impl Args {
    pub fn to_options() -> OptionParser<Self> {
        construct!(Self {
            output(),
            prologue(),
            output_line_suffix(),
            markers(),
            files(),
        })
        .to_options()
    }
}

fn files() -> impl Parser<Vec<FileArg>> {
    positional::<FileArg>("FILE").some("Must specify at least one FILE!")
}

/// Single output destination (`-` for stdout, otherwise a path). If omitted, each
/// FILE is updated in place (or stdin → stdout).
fn output() -> impl Parser<Option<FileArg>> {
    long("output")
        .short('o')
        .argument::<FileArg>("DEST")
        .optional()
}

/// Statements to execute before running CogShell scripts. Executed once per file before
/// the first block.
fn prologue() -> impl Parser<Vec<String>> {
    long("prologue").argument("LINE").many()
}

/// Suffix to append to each output line
fn output_line_suffix() -> impl Parser<String> {
    long("suffix")
        .short('s')
        .argument("SUFFIX")
        .fallback(String::new())
}

/// The patterns surrounding cog inline instructions. Should include three
/// values separated by spaces, the start, end, and end-output markers.
fn markers() -> impl Parser<MarkerDefs> {
    long("markers")
        .argument::<MarkerDefs>("START END END-OUTPUT")
        .fallback(MarkerDefs::from_str("[[[cogsh ]]] [[[end]]]").unwrap())
        .display_fallback()
}

use std::{
    convert::Infallible,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader, Write, stdin, stdout},
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

use atomic_writer::AtomicFileWriter;
use bpaf::{OptionParser, Parser, construct, long, positional};

/// A "path" to read or write from (where `-` means stdin or stdout)
#[derive(Debug)]
pub enum Pipe {
    File(PathBuf),
    Stream,
}

impl Pipe {
    pub fn with_dir(&self, dir: PipeDir) -> PipeWithDir {
        PipeWithDir(self, dir)
    }
}

#[derive(Debug)]
pub enum PipeDir {
    Read,
    Write,
}

#[derive(Debug)]
pub struct PipeWithDir<'a>(pub &'a Pipe, pub PipeDir);

impl Display for PipeWithDir<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self(Pipe::File(path), _) => {
                let pfx = match path.as_os_str().as_encoded_bytes() {
                    b"<stdin>" | b"<stdout>" => "./",
                    _ => "",
                };
                write!(f, "{}{}", pfx, path.display())
            }
            Self(Pipe::Stream, PipeDir::Read) => write!(f, "<stdin>"),
            Self(Pipe::Stream, PipeDir::Write) => write!(f, "<stdout>"),
        }
    }
}

impl FromStr for Pipe {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(Self::Stream)
        } else {
            Ok(Self::File(PathBuf::from(s)))
        }
    }
}

impl<'a> Pipe {
    pub fn open_for_read(&'a self) -> io::Result<Box<dyn BufRead + 'a>> {
        match self {
            Self::Stream => Ok(Box::new(BufReader::new(stdin()))),
            Self::File(path) => Ok(Box::new(File::open_buffered(path)?)),
        }
    }
    pub fn open_for_write(&'a self) -> io::Result<Box<dyn Write + 'a>> {
        match self {
            Self::Stream => Ok(Box::new(stdout())),
            Self::File(path) => Ok(Box::new(AtomicFileWriter::new(path))),
        }
    }
}

/// Enum to restrict combinations of source/dest args. (Output destination can only be
/// specified if there is exactly one source arg.)
#[derive(Debug)]
pub enum SourceAndDestArgs {
    /// Process any number of files in-place
    SourcesInPlace(Vec<Pipe>),
    /// Process exactly one input (stdin or file) to one output (stdout or file)
    SingleSourceAndDest(Pipe, Pipe),
}

fn source() -> impl Parser<Pipe> {
    positional::<Pipe>("SOURCE")
}

fn dest() -> impl Parser<Pipe> {
    long("output").short('o').argument::<Pipe>("DEST")
}

fn source_and_dest() -> impl Parser<SourceAndDestArgs> {
    construct!(SourceAndDestArgs::SingleSourceAndDest(source(), dest()))
}

fn sources() -> impl Parser<Vec<Pipe>> {
    source().some("Must specify at least one SOURCE!")
}

fn sources_in_place() -> impl Parser<SourceAndDestArgs> {
    construct!(SourceAndDestArgs::SourcesInPlace(sources()))
}

fn source_and_dest_args() -> impl Parser<SourceAndDestArgs> {
    construct!([source_and_dest(), sources_in_place()])
}

#[derive(Clone, Debug)]
pub struct MarkerDefs([String; 3]);

impl Deref for MarkerDefs {
    type Target = [String; 3];
    fn deref(self: &MarkerDefs) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MarkerDefs {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split_ascii_whitespace().collect();
        match parts.as_array::<3>() {
            Some(arr) => Ok(Self(arr.map(String::from))),
            None => Err(format!(
                "Marker config `{s}` does not consist of three whitespace-separated words!"
            )),
        }
    }
}

impl Display for MarkerDefs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c] = &self.0;
        write!(f, "{a} {b} {c}")
    }
}

/// The patterns surrounding cog inline instructions. Should include three
/// values separated by spaces, the start, end, and end-output markers.
fn markers() -> impl Parser<MarkerDefs> {
    long("markers")
        .argument::<MarkerDefs>("START END END-OUTPUT")
        .fallback(MarkerDefs::from_str("[[[cogsh ]]] [[[end]]]").unwrap())
        .display_fallback()
}

#[derive(Debug)]
pub struct Args {
    pub source_and_dest: SourceAndDestArgs,
    pub prologue: Vec<String>,
    pub output_line_suffix: String,
    pub markers: MarkerDefs,
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

impl Args {
    pub fn to_options() -> OptionParser<Self> {
        construct!(Self {
            source_and_dest(),
            prologue(),
            output_line_suffix(),
            markers(),
        })
        .to_options()
    }
}

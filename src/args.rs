use std::{
    any::TypeId,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write, stdin, stdout},
    marker::PhantomData,
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

use bpaf::{Parser, construct, long, positional};

use crate::utils::atomic_writer::AtomicFileWriter;

/// A "path" to read or write from (where `-` means stdin or stdout)
pub enum Pipe {
    File(PathBuf),
    Stream,
}

pub enum PipeDir {
    Read,
    Write,
}

pub struct PipeWithDir(Pipe, PipeDir);

impl Display for PipeWithDir {
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
    type Err = !;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(Self::Stream)
        } else {
            Ok(Self::File(PathBuf::from(s)))
        }
    }
}

impl Pipe {
    pub fn open_for_read(&self) -> io::Result<Box<dyn BufRead>> {
        match self {
            Self::Stream => Ok(Box::new(BufReader::new(stdin()))),
            Self::File(path) => Ok(Box::new(File::open_buffered(path)?)),
        }
    }
    pub fn open_for_write(self) -> io::Result<Box<dyn Write>> {
        match self {
            Self::Stream => Ok(Box::new(stdout())),
            Self::File(path) => Ok(Box::new(AtomicFileWriter::new(path))),
        }
    }
}

/// Enum to restrict combinations of source/dest args. (Output destination can only be
/// specified if there is exactly one source arg.)
pub enum SourceAndDestArgs {
    /// Process any number of files in-place
    SourcesInPlace(Vec<Pipe>),
    /// Process exactly one input (stdin or file) to one output (stdout or file)
    SourceAndDest(Pipe, Pipe),
}

fn source() -> impl Parser<Pipe> {
    positional::<Pipe>("SOURCE")
}

fn dest() -> impl Parser<Pipe> {
    long("output").short('o').argument::<Pipe>("DEST")
}

fn source_and_dest() -> impl Parser<SourceAndDestArgs> {
    construct!(SourceAndDestArgs::SourceAndDest(source(), dest()))
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

#[derive(Clone)]
pub struct MarkerConfig([String; 3]);

impl Deref for MarkerConfig {
    type Target = [String; 3];
    fn deref(self: &MarkerConfig) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MarkerConfig {
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

impl Display for MarkerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c] = &self.0;
        write!(f, "{a} {b} {c}")
    }
}

/// The patterns surrounding cog inline instructions. Should include three
/// values separated by spaces, the start, end, and end-output markers.
fn markers() -> impl Parser<MarkerConfig> {
    long("markers")
        .argument::<MarkerConfig>("START END END-OUTPUT")
        .fallback(MarkerConfig::from_str("[[[cogsh ]]] [[[end]]]").unwrap())
        .display_fallback()
}

pub struct Args {
    pub source_and_dest: SourceAndDestArgs,
    pub prologue: Vec<String>,
    pub output_line_suffix: String,
    pub markers: MarkerConfig,
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
    pub fn parser() -> impl Parser<Self> {
        construct!(Self {
            source_and_dest(),
            prologue(),
            output_line_suffix(),
            markers(),
        })
    }
}

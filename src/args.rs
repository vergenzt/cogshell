use std::{
    fs::File,
    io::{self, BufReader, stdin, stdout},
    marker::PhantomData,
    ops::Deref,
    path::PathBuf,
};

use bpaf::{Parser, construct, long, positional};

use crate::utils::atomic_writer::AtomicFileWriter;

trait BufReadPlusWrite {}
impl<T: io::BufRead + io::Write> BufReadPlusWrite for Box<T> {}

pub trait ReadOrWrite {}
pub type Read = Box<dyn io::BufRead>;
pub type Write = Box<dyn io::Write>;
pub type ReadAndWrite = Box<dyn BufReadPlusWrite>;

impl ReadOrWrite for Read {}
impl ReadOrWrite for Write {}
impl ReadOrWrite for ReadAndWrite {}

/// A "path" to read or write from (where `-` means stdin or stdout)
pub enum FileOrStream<RW: ReadOrWrite> {
    File(PathBuf, PhantomData<RW>),
    Stream(PhantomData<RW>),
}

impl FileOrStream<_> {
  pub fn to_string()
}

impl<RW: ReadOrWrite> FileOrStream<RW> {
    /// Interpret exactly `-` as std{in/out}, anything else as itself.
    pub fn parse(path: PathBuf) -> Self {
        match path {
            [b'-'] => Self::Stream(PhantomData),
            _ => Self::File(path, PhantomData),
        }
    }
}

impl FileOrStream<Read> {
    pub fn open(&self) -> Read {
        match self {
            Self::Stream(_) => Box::new(BufReader::new(stdin())),
            Self::File(path, _) => Box::new(File::open_buffered(path)?),
        }
    }
}

impl FileOrStream<Write> {
    pub fn open(self) -> Write {
        match self {
            Self::Stream(_) => Box::new(stdout()),
            Self::File(path, _) => Box::new(AtomicFileWriter::new(path)),
        }
    }
}

/// Enum to restrict combinations of source/dest args. (Output destination can only be
/// specified if there is exactly one source arg.)
pub enum SourceAndDestArgs {
    /// Process any number of files in-place
    SourcesInPlace(Vec<FileOrStream<ReadAndWrite>>),
    /// Process exactly one input (stdin or file) to one output (stdout or file)
    SourceAndDest(FileOrStream<Read>, FileOrStream<Write>),
}

fn source() -> impl Parser<FileOrStream<Read>> {
    positional::<PathBuf>("SOURCE").map(FileOrStream::parse)
}

fn dest() -> impl Parser<FileOrStream<Write>> {
    long("output")
        .short('o')
        .argument::<PathBuf>("DEST")
        .map(FileOrStream::parse)
}

fn source_and_dest() -> impl Parser<SourceAndDestArgs> {
    construct!(SourceAndDestArgs::SourceAndDest(source(), dest()))
}

fn sources() -> impl Parser<Vec<FileOrStream<ReadAndWrite>>> {
    source().some("Must specify at least one SOURCE!")
}

fn sources_in_place() -> impl Parser<SourceAndDestArgs> {
    construct!(SourceAndDestArgs::SourcesInPlace(sources()))
}

fn source_and_dest_args() -> impl Parser<SourceAndDestArgs> {
    construct!([source_and_dest(), sources_in_place()])
}

pub struct MarkerConfig([String; 3]);

impl MarkerConfig {
    fn parse(arg: String) -> Result<Self, String> {
        let parts: Vec<_> = arg.split_ascii_whitespace().collect();
        match parts.as_array::<3>() {
            Some(arr) => Ok(Self(arr.map(String::from))),
            None => {
                let disp = arg.display();
                Err(format!(
                    "Marker config `{disp}` does not consist of three whitespace-separated words!"
                ))
            }
        }
    }
}

/// The patterns surrounding cog inline instructions. Should include three
/// values separated by spaces, the start, end, and end-output markers.
fn marker() -> impl Parser<MarkerConfig> {
    long("markers")
        .argument::<&str>("START END END-OUTPUT")
        .fallback("[[[cogsh ]]] [[[end]]]")
        .parse(MarkerConfig::parse)
        .display_fallback()
}

impl Deref for MarkerConfig {
    type Target = [String; 3];
    fn deref(self: &MarkerConfig) -> &Self::Target {
        &self.0
    }
}

pub struct Args {
    pub source_and_dest: SourceAndDestArgs,
    pub prologue: Vec<String>,
    pub output_line_suffix: Option<String>,
    pub markers: MarkerConfig,
}

/// Statements to execute before running CogShell scripts. Executed once per file before
/// the first block.
fn prologue() -> impl Parser<Vec<String>> {
    long("prologue").argument("LINE").many()
}

/// Suffix to append to each output line
fn output_line_suffix() -> impl Parser<Option<String>> {
    long("suffix").short('s').argument("SUFFIX")
}

impl Args {
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

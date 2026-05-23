use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    fs,
    io::{self, Read as _},
    marker::PhantomData,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    str::FromStr,
};

use atomic_writer::AtomicFileWriter;

use crate::deref_field;

pub type Read = Box<dyn io::BufRead>;
pub type Write = Box<dyn io::Write>;

pub trait ReadOrWrite {
    const STREAM_LABEL: &[u8];
}

impl ReadOrWrite for Read {
    const STREAM_LABEL: &[u8] = b"<stdin>";
}
impl ReadOrWrite for Write {
    const STREAM_LABEL: &[u8] = b"<stdout>";
}

/// A reference of unspecified read/write status, which is either `-` for stdin/stdout, or
/// a file path.
#[derive(Debug, Clone)]
pub enum FileArg {
    Stream,
    OnDisk(PathBuf),
}

impl FromStr for FileArg {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "-" {
            Self::Stream
        } else {
            Self::OnDisk(PathBuf::from(s))
        })
    }
}

pub struct File<RW: ReadOrWrite> {
    /// The argument string representing the file as provided on the command line
    pub arg: FileArg,
    /// PhantomData reference to record that this struct's behavior does depend on the RW param
    pd: PhantomData<RW>,
}

deref_field! {
  impl<RW: ReadOrWrite> *File<RW> = .arg: FileArg
}

impl<RW: ReadOrWrite> From<&FileArg> for File<RW> {
    fn from(arg: &FileArg) -> Self {
        let arg = arg.clone();
        let pd = PhantomData;
        Self { arg, pd }
    }
}

impl<RW: ReadOrWrite> Display for File<RW> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.arg {
            FileArg::OnDisk(path) => {
                f.write_str(match path.as_os_str().as_bytes() {
                    Read::STREAM_LABEL | Write::STREAM_LABEL => "./",
                    _ => "",
                })?;
                f.write_str(&path.to_string_lossy())?;
            }
            FileArg::Stream => f.write_str(&String::from_utf8_lossy(RW::STREAM_LABEL))?,
        }
        Ok(())
    }
}

impl<RW: ReadOrWrite> Debug for File<RW> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl File<Read> {
    /// Reads the full contents of the file into memory.
    pub fn read(&mut self) -> io::Result<String> {
        let mut content = String::new();
        match &self.arg {
            FileArg::Stream => {
                io::stdin().lock().read_to_string(&mut content)?;
            }
            FileArg::OnDisk(path) => {
                fs::File::open(path)?.read_to_string(&mut content)?;
            }
        }
        Ok(content)
    }
}

impl File<Write> {
    /// Open this file (or stdout) for writing.
    pub fn open(&self) -> io::Result<Box<dyn io::Write>> {
        match &self.arg {
            FileArg::Stream => Ok(Box::new(io::stdout())),
            FileArg::OnDisk(path) => Ok(Box::new(AtomicFileWriter::new(path.clone())?)),
        }
    }
}

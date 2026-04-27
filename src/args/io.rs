use std::{
    ffi::OsStr,
    fs::File,
    io::{self},
    path::PathBuf,
    str::FromStr,
};

// https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
mod private {
    pub trait SealedInOrOut {}
    impl SealedInOrOut for super::In {}
    impl SealedInOrOut for super::Out {}
}

#[derive(Debug, Clone)]
pub struct In;

#[derive(Debug, Clone)]
pub struct Out;

pub trait InOrOut: private::SealedInOrOut {
    type IOBox;
    fn inst() -> Self;
    const STREAM_LABEL: &'static str;
    fn open_stream() -> Self::IOBox;
    fn open_file(file: &PathBuf) -> io::Result<Self::IOBox>;
}

macro_rules! impl_in_or_out {
    ($struct:path, $rw:path, $streamname:ident, $openfile:path) => {
        impl InOrOut for $struct {
            type IOBox = Box<dyn $rw>;
            fn inst() -> Self {
                $struct
            }
            const STREAM_LABEL: &'static str = concat!("<", stringify!($streamname), ">");
            fn open_stream() -> Self::IOBox {
                Box::new(io::$streamname())
            }
            fn open_file(file: &PathBuf) -> io::Result<Self::IOBox> {
                Ok(Box::new($openfile(file)?))
            }
        }
    };
}

impl_in_or_out!(In, io::Read, stdin, File::open);
impl_in_or_out!(Out, io::Write, stdout, File::create);

/// Represents an argument which can be either a file path or "-" for stdin/stdout.
/// The generic parameter `IO` is bound by `InOrOut` to be either `::In` or `::Out`.
#[derive(Debug, Clone)]
pub enum FileOrStream<IO: InOrOut> {
    File(PathBuf, IO),
    Stream(IO),
}

impl<IO: InOrOut> FileOrStream<IO> {
    pub fn file(path: PathBuf) -> Self {
        Self::File(path, IO::inst())
    }

    pub fn stream() -> Self {
        Self::Stream(IO::inst())
    }

    /// Get a name for this stream suitable for display
    pub fn to_str(&self) -> &str {
        match self {
            FileOrStream::File(path_buf, _) => path_buf.to_str().unwrap(),
            FileOrStream::Stream(_) => IO::STREAM_LABEL,
        }
    }

    /// Get name for this stream for env var value
    pub fn to_os_str(&self) -> &OsStr {
        match self {
            FileOrStream::File(path_buf, _) => path_buf.as_os_str(),
            FileOrStream::Stream(_) => OsStr::new(IO::STREAM_LABEL),
        }
    }

    /// Get a handle to read/write the input/output
    pub fn open(&self) -> io::Result<IO::IOBox> {
        match self {
            Self::File(path, _) => IO::open_file(path),
            Self::Stream(_) => Ok(IO::open_stream()),
        }
    }
}

impl<IO: InOrOut> FromStr for FileOrStream<IO> {
    type Err = !;

    /// Convert "-" into stdin or stdout, depending on stream type
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "-" {
            Self::Stream(IO::inst())
        } else {
            Self::File(PathBuf::from(s), IO::inst())
        })
    }
}

// impl<IO: InOrOut> Display for FileOrStream<IO> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Stream(_) => f.write_str(IO::STREAM_LABEL),
//             Self::File(path_buf, _) => {
//                 let path = path_buf.to_str().unwrap();
//                 match path {
//                     "-" | In::STREAM_LABEL | Out::STREAM_LABEL => {
//                         f.write_str("./");
//                     }
//                     _ => (),
//                 }
//                 path.fmt(f)
//             }
//         }
//     }
// }

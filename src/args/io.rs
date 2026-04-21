use std::{
    convert::Infallible,
    fmt::Display,
    fs::{self},
    io::{self, BufReader, BufWriter, stdin, stdout},
    marker::PhantomData,
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

// https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
macro_rules! sealed_trait {
  (
    $(#[$attr:meta])*
    $vis:vis trait $name:ident
    = (
      $(
        $impl:ident
        $(< $($impl_gen:ident $(: $impl_bnd:ident )? ),+ >)?
      )|+ $(|)?
    )
    $(< $($gen:ident $( : $bnd:ident )? ),+ > )?
    { $($body:tt)* }
  ) => {
        #[allow(non_snake_case)]
        mod ${concat(private, $name)} {
          pub trait ${concat(Sealed, $name)} {}
          $(
            impl
            $(< $($impl_gen $(: super::$impl_bnd)? ) ,+ >)?
            ${concat(Sealed, $name)} for super::$impl
            $(<$($impl_gen),+>)?
            {}
          )+
        }

        $(#[$attr])*
        $vis trait $name$(<$($gen$(: $bnd)?),+>)? : ${concat(private, $name)}::${concat(Sealed, $name)} {
          $($body)*
        }
    };
}

sealed_trait! {
    /// Sealed trait (In or Out) representing data flow direction
    pub trait InOrOut = (In | Out) {
        type IOBox;
        fn inst() -> Self;
        const STREAM_LABEL: &'static str;
        const STREAM_TYPE: Stream;
        fn open_stream() -> Self::IOBox;
        fn open_file(file: &PathBuf) -> io::Result<Self::IOBox>;
    }
}

#[derive(Debug, Clone)]
pub struct In;

#[derive(Debug, Clone)]
pub struct Out;

macro_rules! impl_in_or_out {
    ($struct:path, $rw:path, $streamname:ident, $enumname:ident, $openfile:path) => {
        impl InOrOut for $struct {
            type IOBox = Box<dyn $rw>;
            fn inst() -> Self {
                $struct
            }
            const STREAM_LABEL: &'static str = concat!("<", stringify!($streamname), ">");
            const STREAM_TYPE: Stream = Stream::$enumname;
            fn open_stream() -> Self::IOBox {
                Box::new(io::$streamname())
            }
            fn open_file(file: &PathBuf) -> io::Result<Self::IOBox> {
                Ok(Box::new($openfile(file)?))
            }
        }
    };
}

impl_in_or_out!(In, io::Read, stdin, Stdin, fs::File::open);
impl_in_or_out!(Out, io::Write, stdout, Stdout, fs::File::create);

sealed_trait! {
  /// Represents an argument which can be either a file path or "-" for stdin/stdout.
  /// The generic parameter `IO` is bound by `InOrOut` to be either `::In` or `::Out`.
  pub trait FileOrStream = (File<IO: InOrOut> | Stream) <IO: InOrOut> {
    /// Get a handle to read/write the input/output
    fn open(&self) -> io::Result<IO::IOBox>;

    fn from_str(s: &str) -> impl Self {
      match s {
        "-" => IO::STREAM_TYPE,
        _ => PathBuf::from(s),
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct File<IO: InOrOut> {
    pub path: PathBuf,
    io: IO,
}

impl<IO: InOrOut> FileOrStream<IO> for File<IO> {
    fn open(&self) -> io::Result<IO::IOBox> {
        IO::open_file(&self.path)
    }
}

impl<IO: InOrOut> Deref for File<IO> {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl<IO: InOrOut> From<PathBuf> for File<IO> {
    fn from(path: PathBuf) -> Self {
        let io = IO::inst();
        File { path, io }
    }
}

impl<IO: InOrOut> FromStr for File<IO> {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PathBuf::from_str(s)?.into())
    }
}

impl<IO: InOrOut> Display for File<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.display().fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum Stream {
    Stdout,
    Stdin,
}

impl<IO: InOrOut> FileOrStream<IO> for Stream<IO> {
    fn open(&self) -> io::Result<<IO as InOrOut>::IOBox> {
        Ok(IO::open_stream())
    }
}

impl<IO: InOrOut> Display for Stream<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(IO::STREAM_LABEL)
    }
}

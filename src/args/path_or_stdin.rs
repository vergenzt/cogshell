use std::{
    fmt::Display,
    fs::File,
    io::{self, Read, stdin},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum PathOrStdin {
    Path(PathBuf),
    Stdin,
}

impl PathOrStdin {
    pub fn open(&self) -> io::Result<Box<dyn Read>> {
        Ok(match self {
            Self::Path(path) => Box::new(File::open(path)?),
            Self::Stdin => Box::new(stdin()),
        })
    }
}

impl Display for PathOrStdin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Stdin => "<stdin>",
            Self::Path(path) => {
                // disambiguate
                if path == &PathBuf::from("-") {
                    "./-"
                } else {
                    path.to_str().unwrap()
                }
            }
        })
    }
}

impl FromStr for PathOrStdin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "-" {
            Self::Stdin
        } else {
            Self::Path(PathBuf::from(s))
        })
    }
}

use std::{
    ffi::OsString,
    fmt::{Display, Result},
    ops::Deref,
    os::unix::ffi::OsStringExt,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct MarkerConfig([OsString; 3]);

impl MarkerConfig {
    pub fn parse(arg: OsString) -> Result<Self> {
        let parts: Vec<_> = arg.into_vec().split(|c| c.is_ascii_whitespace()).collect();
        match parts.as_array() {
            Some(arr) => Self(arr.map(OsString::from)),
        }
    }

    pub fn new(markers: [OsString; 3]) -> MarkerConfig {
        for (i, s) in markers.iter().enumerate() {
            // markers cannot contain whitespace
            assert_eq!(s.find(|c| c.is_ascii_whitespace()), None);
            for j in 0..i {
                // markers must be unique
                assert_ne!(markers[j], markers[i]);
            }
        }
        Self(markers)
    }
}

impl Deref for MarkerConfig {
    type Target = [Vec<u8>; 3];
    fn deref(self: &MarkerConfig) -> &Self::Target {
        &self.0
    }
}

impl From<[Vec<u8>; 3]> for MarkerConfig {
    fn from(value: [Vec<u8>; 3]) -> Self {
        MarkerConfig::new(value)
    }
}

impl FromStr for MarkerConfig {
    type Err = String;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split_ascii_whitespace().collect();
        match parts[..] {
            [a, b, c] => Ok(MarkerConfig::new(
                [a, b, c].map(|s| ByteStr::new(s).to_owned()),
            )),
            _ => Err({
                format!("Marker value {str:?} does not contain three whitespace-separated values!")
            }),
        }
    }
}

impl Display for MarkerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c] = &self.0.clone().map(String::from_utf8_lossy_owned);
        write!(f, "{a} {b} {c}")
    }
}

/// Configurable attributes for CogShell execution
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to protect output lines in CogShell'd files from accidental modification by including a hash after the `output_end` marker
    pub output_checksums: bool,
    /// Optional suffix to append to each output line
    pub output_line_suffix: Vec<u8>,
    /// Whether to validate existing checksums following CogShell blocks
    pub verify_input_checksums: bool,
    /// Shell lines which will be prepended to each embedded program before running
    pub prologue: Vec<Vec<u8>>,
    /// Strings indicating start of program, end of program, and end of output
    pub markers: MarkerConfig,
}

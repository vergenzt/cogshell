use std::{fmt::Display, ops::Deref, str::FromStr};

#[derive(Debug, Clone)]
pub struct MarkerConfig([ByteString; 3]);

impl MarkerConfig {
    pub fn new(markers: [ByteString; 3]) -> MarkerConfig {
        for (i, s) in markers.iter().enumerate() {
            // markers cannot contain whitespace
            assert_eq!(s.iter().find(|c| c.is_ascii_whitespace()), None);
            for j in 0..i {
                // markers must be unique
                assert_ne!(markers[j], markers[i]);
            }
        }
        Self(markers)
    }
}

impl Deref for MarkerConfig {
    type Target = [ByteString; 3];
    fn deref(self: &MarkerConfig) -> &Self::Target {
        &self.0
    }
}

impl From<[ByteString; 3]> for MarkerConfig {
    fn from(value: [ByteString; 3]) -> Self {
        MarkerConfig::new(value)
    }
}

impl FromStr for MarkerConfig {
    type Err = String;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = str.split_ascii_whitespace().collect();
        match parts[..] {
            [a, b, c] => Ok(MarkerConfig::new([a, b, c].map(|s| s.into()))),
            _ => Err({
                format!("Marker value {str:?} does not contain three whitespace-separated values!")
            }),
        }
    }
}

impl Display for MarkerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let markers = &self.0;
        markers
            .iter()
            .map(|v| String::from_utf8_lossy_owned(v.clone()))
            .intersperse(String::from(" "))
            .try_for_each(|s| write!(f, "{s}"))
    }
}

/// Configurable attributes for CogShell execution
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to protect output lines in CogShell'd files from accidental modification by including a hash after the `output_end` marker
    pub output_checksums: bool,
    /// Optional suffix to append to each output line
    pub output_line_suffix: ByteString,
    /// Whether to validate existing checksums following CogShell blocks
    pub verify_input_checksums: bool,
    /// Shell lines which will be prepended to each embedded program before running
    pub prologue: Vec<ByteString>,
    /// Strings indicating start of program, end of program, and end of output
    pub markers: MarkerConfig,
}

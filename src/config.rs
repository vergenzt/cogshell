use std::{
    ffi::OsString,
    fmt::{self, Display},
    ops::Deref,
    os::unix::ffi::OsStringExt,
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct MarkerConfig([OsString; 3]);

impl MarkerConfig {
    pub fn parse(arg: OsString) -> Result<Self, fmt::Error> {
        let parts: Vec<_> = arg.into_vec().split(|c| c.is_ascii_whitespace()).collect();
        match parts.as_array() {
            Some(arr) => Self(arr.map(OsString::from)),
            None => todo!(),
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
    type Target = [OsString; 3];
    fn deref(self: &MarkerConfig) -> &Self::Target {
        &self.0
    }
}

impl From<[OsString; 3]> for MarkerConfig {
    fn from(value: [OsString; 3]) -> Self {
        MarkerConfig::new(value)
    }
}

/// Configurable attributes for CogShell execution
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to protect output lines in CogShell'd files from accidental modification by including a hash after the `output_end` marker
    pub output_checksums: bool,
    /// Optional suffix to append to each output line
    pub output_line_suffix: OsString,
    /// Whether to validate existing checksums following CogShell blocks
    pub verify_input_checksums: bool,
    /// Shell lines which will be prepended to each embedded program before running
    pub prologue: Vec<OsString>,
    /// Strings indicating start of program, end of program, and end of output
    pub markers: MarkerConfig,
}

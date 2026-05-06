use std::{
    ffi::OsString,
    fmt::{self, Display},
    ops::Deref,
    os::unix::ffi::OsStringExt,
    str::FromStr,
};

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

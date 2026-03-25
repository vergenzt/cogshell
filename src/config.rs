use std::ops::Deref;

use enum_primitive_derive::Primitive;

/// Names for the environment variables CogShell will provide to embedded programs
#[derive(Debug)]
pub struct EnvConfig<'a> {
    /// Absolute path to the file containing current embedded CogShell program
    source_file: &'a str,
    /// Path to tempfile containing the previous output of current CogShell block
    prev_output: &'a str,
    /// Path to tempfile to which the *next* output of current CogShell block should be written (if not using stdout)
    next_output: &'a str,
}

impl Default for EnvConfig<'_> {
    fn default() -> Self {
        Self {
            source_file: "THIS",
            prev_output: "OUTPUT_PREV",
            next_output: "OUTPUT",
        }
    }
}

#[derive(Copy, Clone, Debug, Primitive)]
pub enum MarkerKind {
    // (note: order matters; discriminant used as index into parser state vec.)
    ProgramStart = 0,
    ProgramEnd = 1,
    OutputEnd = 2,
}

impl From<MarkerKind> for &'static str {
    fn from(value: MarkerKind) -> Self {
        match value {
            MarkerKind::ProgramStart => "program start",
            MarkerKind::ProgramEnd => "program end",
            MarkerKind::OutputEnd => "output end",
        }
    }
}

#[derive(Debug)]
pub struct MarkerConfig<'a>(pub [&'a str; 3]);

impl<'a> Deref for MarkerConfig<'a> {
    type Target = [&'a str; 3];
    fn deref(self: &MarkerConfig<'a>) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<[&'a str; 3]> for MarkerConfig<'a> {
    fn from(value: [&'a str; 3]) -> Self {
        MarkerConfig(value)
    }
}

impl Default for MarkerConfig<'_> {
    fn default() -> Self {
        Self(["[[[cogsh ", "]]]", "[[[end]]]"])
    }
}

/// Configurable attributes for CogShell execution
#[derive(Debug)]
pub struct Config<'a> {
    /// Whether to protect output lines in CogShell'd files from accidental modification by including a hash after the `output_end` marker
    pub output_checksum: bool,
    /// Optional suffix to append to each output line
    pub output_line_suffix: &'a str,
    /// Shell lines which will be prepended to each embedded program before running
    pub prologue: Vec<&'a str>,
    /// Array of marker strings indicating start of program, end of program, and end of output
    pub markers: MarkerConfig<'a>,
    /// Overrides for names of CogShell environment variables
    pub env_names: EnvConfig<'a>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            output_checksum: true,
            output_line_suffix: "",
            prologue: Vec::new(),
            markers: MarkerConfig::default(),
            env_names: EnvConfig::default(),
        }
    }
}

use std::ops::Deref;

/// Names for the environment variables CogShell will provide to embedded programs
#[derive(Debug)]
pub struct EnvConfig<'a> {
    /// Absolute path to the source file containing current embedded CogShell program
    pub source_file_path: &'a str,
    /// Line number within the source file where current program source starts
    pub prog_start_line: &'a str,
    /// Column number on the first line the source file where current program source starts
    pub prog_start_col: &'a str,
    /// Byte offset from start of file to where current program source starts
    pub prog_start_offset: &'a str,
    /// Path to tempfile containing the previous output of current CogShell block
    pub output_prev: &'a str,
    /// Path to tempfile to which the *next* output of current CogShell block should be written (if not using stdout)
    pub output_next: &'a str,
}

impl Default for EnvConfig<'_> {
    fn default() -> Self {
        Self {
            source_file_path: "SOURCE",
            prog_start_line: "SOURCE_PROG_START_LINE",
            prog_start_col: "SOURCE_PROG_START_COL",
            prog_start_offset: "SOURCE_PROG_START_OFFSET",
            output_prev: "OUTPUT_PREV",
            output_next: "OUTPUT",
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
    pub output_checksums: bool,
    /// Optional suffix to append to each output line
    pub output_line_suffix: &'a str,
    /// Whether to validate existing checksums following CogShell blocks
    pub verify_input_checksums: bool,
    /// Shell lines which will be prepended to each embedded program before running
    pub prologue: Vec<&'a str>,
    /// Array of marker strings indicating start of program, end of program, and end of output
    pub marker_strings: MarkerConfig<'a>,
    /// Overrides for names of CogShell environment variables
    pub env_var_names: EnvConfig<'a>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            output_checksums: true,
            output_line_suffix: "",
            verify_input_checksums: true,
            prologue: Vec::new(),
            marker_strings: MarkerConfig::default(),
            env_var_names: EnvConfig::default(),
        }
    }
}

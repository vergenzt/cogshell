pub const LINE_SEPS: &[&[u8]] = &[b"\r\n", b"\n"];

/// Names for the environment variables CogShell will provide to embedded programs
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

/// Configurable attributes for CogShell execution
pub struct Config<'a> {
    /// Whether to protect output lines in CogShell'd files from accidental modification by including a hash after the `output_end` marker
    pub output_checksum: bool,
    /// Optional suffix to append to each output line
    pub output_line_suffix: &'a str,
    /// Shell lines which will be prepended to each embedded program before running
    pub prologue: Vec<&'a str>,
    /// Array of marker strings indicating start of program, end of program, and end of output
    pub markers: [&'a str; 3],
    /// Overrides for names of CogShell environment variables
    pub env_names: EnvConfig<'a>,
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            output_checksum: true,
            output_line_suffix: "",
            prologue: Vec::new(),
            markers: ["[[[cogsh ", "]]]", "[[[end]]]"],
            env_names: EnvConfig::default(),
        }
    }
}

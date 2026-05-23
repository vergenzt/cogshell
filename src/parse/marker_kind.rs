use std::fmt::Display;

use regex::CaptureLocations;

/// A kind of marker
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MarkerKind {
    // (note: order matters; discriminant used as index into parser state vec.)
    /// Indicates the start of an embedded CogShell program
    ProgramStart = 0,
    /// Indicates the end of an embedded CogShell program, and the start of its output
    ProgramEnd,
    /// Indicates the end of an embedded CogShell program's output
    OutputEnd,
}

impl MarkerKind {
    pub const ALL: [Self; 3] = [Self::ProgramStart, Self::ProgramEnd, Self::OutputEnd];

    pub fn description(self: MarkerKind) -> &'static str {
        match self {
            MarkerKind::ProgramStart => "program start marker",
            MarkerKind::ProgramEnd => "program end marker",
            MarkerKind::OutputEnd => "output end marker",
        }
    }
}

impl From<&CaptureLocations> for MarkerKind {
    fn from(caps: &CaptureLocations) -> Self {
        // markers_re contains 3 capture groups, one for each marker kind
        let kind_idx = (0..3)
            .find(|grp_idx| {
                // find index of the first non-None capture group
                caps.get(grp_idx + 1).is_some()
            })
            .unwrap();
        Self::ALL[kind_idx]
    }
}

impl Display for MarkerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

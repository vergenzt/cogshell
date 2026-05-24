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

    /// Used by error reporting.
    pub fn description(self: MarkerKind) -> &'static str {
        match self {
            MarkerKind::ProgramStart => "program start marker",
            MarkerKind::ProgramEnd => "program end marker",
            MarkerKind::OutputEnd => "output end marker",
        }
    }

    /// See [`IsInlineOk`]. Called by [`super::marker_inst::MarkerInst::ok_after`].
    pub fn inline_ok(self: &MarkerKind) -> IsInlineOk {
        match self {
            MarkerKind::ProgramStart => IsInlineOk::MustFollowOnSeparateLine,
            MarkerKind::ProgramEnd => IsInlineOk::InlineOk,
            MarkerKind::OutputEnd => IsInlineOk::MustFollowOnSeparateLine,
        }
    }
}

/// Bool-equivalent enum describing whether a marker kind is allowed to appear on the same
/// line as a marker preceding it (whether in the same block or a previous one).
///
/// Used by [`MarkerKind::inline_ok`].
#[derive(PartialEq)]
pub enum IsInlineOk {
    MustFollowOnSeparateLine,
    InlineOk,
}

impl From<IsInlineOk> for bool {
    fn from(value: IsInlineOk) -> bool {
        value == IsInlineOk::InlineOk
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

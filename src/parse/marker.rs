use std::fmt::Display;

use regex::Match;

pub struct Marker<'a> {
    /// The marker itself, including the start & end locations within original content
    pub span: Match<'a>,
    /// The line number of the marker start point
    pub line: usize,
    /// The column number of the marker start point
    pub col: usize,
}

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
    pub fn description(self: MarkerKind) -> &'static str {
        match self {
            MarkerKind::ProgramStart => "program start marker",
            MarkerKind::ProgramEnd => "program end marker",
            MarkerKind::OutputEnd => "output end marker",
        }
    }
}

impl From<usize> for MarkerKind {
    fn from(i: usize) -> MarkerKind {
        [
            MarkerKind::ProgramStart,
            MarkerKind::ProgramEnd,
            MarkerKind::OutputEnd,
        ][i]
    }
}

impl Display for MarkerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

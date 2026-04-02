use std::fmt::Display;

use regex::Match;

#[derive(Copy, Clone, Debug)]
pub enum MarkerKind {
    // (note: order matters; discriminant used as index into parser state vec.)
    ProgramStart = 0,
    ProgramEnd,
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

impl Into<MarkerKind> for usize {
    fn into(self) -> MarkerKind {
        [
            MarkerKind::ProgramStart,
            MarkerKind::ProgramEnd,
            MarkerKind::OutputEnd,
        ][self]
    }
}

impl Display for MarkerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

#[derive(Clone, Debug)]
pub struct Marker<'a> {
    pub kind: MarkerKind,
    pub span: Match<'a>,
}

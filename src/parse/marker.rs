use std::fmt::Display;

use regex::Match;

use crate::parse::{Loc, Span};

#[derive(Debug, Clone, Copy)]
pub struct MarkerInst<'a> {
    /// The content this marker was matched from
    content: &'a str,
    /// The span within the content where the marker was found
    pub span: Span,
}

impl<'a> MarkerInst<'a> {
    pub fn new(content: &'a str, mat: Match<'a>, line: usize, line_start: usize) -> Self {
        let start = {
            let offset = mat.range().start;
            let line = line;
            let col = offset - line_start;
            Loc { offset, line, col }
        };
        let end = start + mat.as_str();
        let span = Span { start, end };
        Self { content, span }
    }

    pub fn bytes(&self) -> &'a str {
        &self.content[*self.span.start..*self.span.end]
    }
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
    pub const ALL: [Self; 3] = [Self::ProgramStart, Self::ProgramEnd, Self::OutputEnd];

    pub fn description(self: MarkerKind) -> &'static str {
        match self {
            MarkerKind::ProgramStart => "program start marker",
            MarkerKind::ProgramEnd => "program end marker",
            MarkerKind::OutputEnd => "output end marker",
        }
    }
}

impl Display for MarkerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

use std::{bstr::ByteStr, fmt::Display};

use crate::parse::Span;

#[derive(Debug, Clone, Copy)]
pub struct MarkerInst<'a> {
    /// The content this marker was matched from
    content: &'a [u8],
    /// The span within the content where the marker was found
    pub span: Span,
}

impl<'a> MarkerInst<'a> {
    pub fn new(content: &'a [u8], span: Span) -> Self {
        Self { content, span }
    }

    pub fn bytes(&self) -> &'a [u8] {
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

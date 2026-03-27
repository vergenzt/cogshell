use std::{
    fmt::Display,
    ops::{Deref, Range},
};

use crate::source::{location::SourceLocation, span::SourceSpan};

#[derive(Copy, Clone, Debug)]
pub enum MarkerKind {
    // (note: order matters; discriminant used as index into parser state vec.)
    ProgramStart = 0,
    ProgramEnd = 1,
    OutputEnd = 2,
}

impl MarkerKind {
    pub const ALL: [Self; 3] = [Self::ProgramStart, Self::ProgramEnd, Self::OutputEnd];

    pub fn description(self: MarkerKind) -> &'static str {
        match self {
            MarkerKind::ProgramStart => "program start",
            MarkerKind::ProgramEnd => "program end",
            MarkerKind::OutputEnd => "output end",
        }
    }
}

impl Display for MarkerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.description())
    }
}


#[derive(Clone, Debug)]
pub struct SourceMarker<'a> {
    pub kind: MarkerKind,
    pub span: SourceSpan<'a>,
    pub preceding_line_starts: Vec<SourceLocation>,
}

impl<'a> Deref for SourceMarker<'a> {
    type Target = SourceSpan<'a>;

    fn deref(&self) -> &Self::Target {
        &self.span
    }
}

impl Display for SourceMarker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.span.content[self.span.start.offset..self.span.end.offset])
    }
}

impl From<&SourceMarker<'_>> for Range<usize> {
    fn from(value: &SourceMarker) -> Self {
        value.span.start.offset..value.span.end.offset
    }
}

impl<'a> From<&SourceMarker<'a>> for &'a str {
    fn from(value: &SourceMarker<'a>) -> Self {
        let range: Range<usize> = value.into();
        &value.span.content[range]
    }
}

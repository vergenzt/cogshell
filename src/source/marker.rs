use std::{
    fmt::Display,
    ops::{Deref, Range},
};

use enum_primitive_derive::Primitive;

use crate::source::{location::SourceLocation, span::SourceSpan};

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
        f.pad(&self.span.content[self.span.start.offset..self.span.end.offset])
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

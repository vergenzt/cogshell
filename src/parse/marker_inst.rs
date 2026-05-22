use regex::Match;

use crate::deref_field;

use super::{loc_line::*, marker_kind::*, span::*};

#[derive(Debug, Clone, Copy)]
pub struct MarkerInst<'s> {
    /// The kind of marker this is
    pub kind: MarkerKind,
    /// The line of content this marker was found in
    pub line: &'s String,
    /// The span within the content where the marker was found
    pub span: Span,
}

deref_field! { impl<'s> *MarkerInst<'s> = .as_str(): str }

impl<'s> MarkerInst<'s> {
    pub fn new(
        kind: MarkerKind,
        mtch: Match<'s>,
        LocatedLine(line_start, line): &LocatedLine<'s>,
    ) -> Self {
        let start = *line_start + &line[..mtch.start()];
        let end = start + mtch.as_str();
        let span = Span { start, end };
        Self { kind, line, span }
    }

    pub fn line(&self) -> &'s String {
        &self.line
    }

    pub fn as_str(&self) -> &'s str {
        &self.line()[self.span.start.col..self.span.end.col]
    }
}

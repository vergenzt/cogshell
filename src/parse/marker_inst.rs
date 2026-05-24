use regex::Match;

use super::{loc_line::*, marker_kind::*, span::*};

#[derive(Debug, Clone, Copy)]
pub struct MarkerInst<'s> {
    /// The kind of marker this is
    pub kind: MarkerKind,
    /// The line of content this marker was found in
    pub line: &'s str,
    /// The span within the content where the marker was found
    pub span: Span,
}

impl<'s> std::ops::Deref for MarkerInst<'s> {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<'s> MarkerInst<'s> {
    pub fn new(
        kind: MarkerKind,
        mtch: Match<'s>,
        &LocatedLine(line_start, line): &LocatedLine<'s>,
    ) -> Self {
        let start = line_start + &line[..mtch.start()];
        let end = start + mtch.as_str();
        let span = Span { start, end };
        Self { kind, line, span }
    }

    /// In upstream `cogapp`, some markers are silently ignored if they appear on the same
    /// line as a preceding marker. This method supports us reporting such cases as errors
    /// instead.
    pub fn ok_after(&self, prev: &MarkerInst) -> bool {
        (self.span.line() == prev.span.line()) == self.kind.inline_ok().into()
    }

    pub fn as_str(&self) -> &'s str {
        let col_start = self.span.start.col;
        let col_end = self.span.end.col;
        &self.line[col_start..col_end]
    }
}

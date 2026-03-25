use std::fmt::Display;
use std::ops::{Deref, Range};

use crate::config::MarkerKind;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl SourceLocation {
    pub fn from(offset: usize, line_start: SourceLocation) -> Self {
        SourceLocation {
            line: line_start.line,
            col: offset - line_start.offset,
            offset,
        }
    }
}

impl Deref for SourceLocation {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.offset
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SourceSpan<'a> {
    pub content: &'a str,
    pub start: SourceLocation,
    pub end: SourceLocation,
    pub line_start: SourceLocation,
}

impl<'a> SourceSpan<'a> {
    pub fn from_range(
        content: &'a str,
        found: Range<usize>,
        line_start: &mut SourceLocation,
    ) -> Self {
        assert_eq!(line_start.col, 0);

        let start = SourceLocation::from(found.start, SourceLocation { ..*line_start });

        // increment line to end of the match
        let lines: Vec<_> = content[found.clone()].split_inclusive('\n').collect();
        line_start.line += lines.len() - 1;
        line_start.offset = found.end - lines.last().unwrap().len();

        let end = SourceLocation::from(found.end, SourceLocation { ..*line_start });

        SourceSpan {
            content,
            start,
            end,
            line_start: *line_start,
        }
    }
}

impl PartialEq for SourceSpan<'_> {
    fn eq(&self, other: &Self) -> bool {
        let [self_addrs, other_addrs] = [self, other].map(|span| {
            let addr = span.content.as_ptr().addr();
            (addr + self.start.offset, addr + self.end.offset)
        });
        self_addrs == other_addrs
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SourceMarker<'a> {
    pub kind: MarkerKind,
    pub span: SourceSpan<'a>,
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

#[cfg(test)]
mod source_span_tests {
    use super::*;

    #[test]
    fn test_source_span_init() {
        let content = "abcd\nefgh\nijkl\n";
        let mut line_start = SourceLocation::default();
        let actual_span = SourceSpan::from_range(content, 1..4, &mut line_start);
        let expected_span = SourceSpan {
            content,
            start: SourceLocation {
                line: 0,
                col: 1,
                offset: 1,
            },
            end: SourceLocation {
                line: 0,
                col: 5,
                offset: 5,
            },
            line_start: SourceLocation::default(),
        };
        assert_eq!(actual_span, expected_span)
    }
}

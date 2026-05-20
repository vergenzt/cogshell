use std::ops::Range;

use crate::parse::Loc;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: Loc,
    pub end: Loc,
}

impl Span {
    pub fn len(&self) -> usize {
        self.end.offset - self.start.offset
    }

    pub fn line(&self) -> usize {
        debug_assert_eq!(self.start.line, self.end.line);
        self.start.line
    }
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Range<usize> {
        value.start.offset..value.end.offset
    }
}

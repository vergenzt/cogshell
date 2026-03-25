use std::ops::Deref;

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

use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use crate::deref_field;

#[derive(Debug, Clone, Copy, Default)]
pub struct Loc {
    /// Byte offset
    pub offset: usize,
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub col: usize,
}

deref_field! { impl *Loc = .offset: usize }

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{}:{}", self.line, self.col)
    }
}

impl Add<&str> for Loc {
    type Output = Loc;

    fn add(self, rhs: &str) -> Self::Output {
        let last_nl = rhs.rfind('\n');
        let nl_count = rhs.bytes().filter(|b| *b == b'\n').count();
        let col = match last_nl {
            // chars after the last newline form the new line; col resets
            Some(nl_pos) => rhs.len() - nl_pos - 1,
            // no newline: continue on the same line
            None => self.col + rhs.len(),
        };
        Loc {
            offset: self.offset + rhs.len(),
            line: self.line + nl_count,
            col,
        }
    }
}

impl Sub<Loc> for Loc {
    type Output = usize;

    fn sub(self, rhs: Loc) -> Self::Output {
        self.offset - rhs.offset
    }
}

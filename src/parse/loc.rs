use std::{
    fmt::Display,
    ops::{Add, Deref, Sub},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Loc {
    /// Byte offset
    pub offset: usize,
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub col: usize,
}

impl Deref for Loc {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.offset
    }
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":{}:{}", self.line, self.col)
    }
}

impl Add<&str> for Loc {
    type Output = Loc;

    fn add(self, rhs: &str) -> Self::Output {
        let (lines, last_line_len) = match rhs
            .as_bytes()
            .iter()
            .enumerate()
            .filter(|(_, c)| **c == b'\n')
            .enumerate()
            .last()
        {
            None => (0, rhs.len()),
            Some((nl_idx, (nl_pos, _))) => (nl_idx + 1, rhs.len() - nl_pos),
        };
        Loc {
            offset: self.offset + rhs.len(),
            line: self.line + lines,
            col: last_line_len,
        }
    }
}

impl Sub<Loc> for Loc {
    type Output = usize;

    fn sub(self, rhs: Loc) -> Self::Output {
        self.offset - rhs.offset
    }
}

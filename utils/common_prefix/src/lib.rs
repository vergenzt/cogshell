#[cfg(test)]
mod test;

use std::{borrow::Borrow, iter::zip};

/// Length of the longest common prefix of characters from the given lines. If there is
/// not more than one line, result is None.
pub fn common_prefix_of_chars<'a>(lines: &[&'a str]) -> Option<&'a str> {
    if lines.len() < 2 {
        return None;
    }
    let mut lines = lines.iter().peekable();
    let mut comm_pfx = *lines.next()?;
    for line in lines {
        let divergence_idx = zip(comm_pfx.bytes(), line.bytes())
            .position(|(c1, c2)| c1 != c2)
            .map(|i| line.floor_char_boundary(i))
            .unwrap_or(0);
        if divergence_idx == 0 {
            return None;
        }
        comm_pfx = &comm_pfx[0..divergence_idx];
    }
    Some(comm_pfx)
}

/// Get any leading (ascii) whitespace at the beginning of the given line
pub fn leading_whitespace<'a>(s: impl Borrow<&'a str>) -> &'a str {
    let s = *s.borrow();
    let i = s.find(|c: char| !c.is_ascii_whitespace()).unwrap_or(0);
    &s[..i]
}

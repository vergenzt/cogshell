use std::{borrow::Borrow, iter::zip};

#[cfg(test)]
mod mod_test;

/// Length of the longest common (fully UTF-8) prefix of characters from the given lines, in bytes.
/// If there is not more than one line, result is None.
pub fn common_prefix_of_chars<'a, Iter, Item>(lines: Iter) -> Option<&'a ByteStr>
where
    Item: Borrow<&'a ByteStr>,
    Iter: IntoIterator<Item = Item>,
{
    let mut lines = lines.into_iter();
    let mut comm_pfx = *lines.next()?.borrow();
    for line in lines {
        let chars = zip(comm_pfx.iter(), line.borrow().iter());
        let in_common = chars.take_while(|(c1, c2)| c1 == c2).count();
        comm_pfx = &comm_pfx[0..in_common];
    }
    Some(comm_pfx)
}

/// Get any leading (ascii) whitespace at the beginning of the given &ByteStr
pub fn leading_whitespace<'a>(s: impl Borrow<&'a ByteStr>) -> &'a ByteStr {
    let s = s.borrow();
    match s
        .iter()
        .enumerate()
        .take_while(|(_i, c)| c.is_ascii_whitespace())
        .last()
    {
        Some((last_ws_idx, _)) => &s[..last_ws_idx + 1],
        None => &[],
    }
}

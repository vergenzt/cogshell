use std::{borrow::Borrow, iter::zip};

#[cfg(test)]
mod mod_test;

/// Length of the longest common (fully UTF-8) prefix of characters from the given lines, in bytes.
/// If there is not more than one line, result is None.
pub fn common_prefix_of_chars<'a, Iter, Item>(lines: Iter) -> Option<&'a str>
where
    Item: Borrow<&'a str>,
    Iter: IntoIterator<Item = Item>,
{
    let mut lines = lines.into_iter();
    let mut comm_pfx = *lines.next()?.borrow();
    for line in lines {
        let chars = zip(comm_pfx.chars(), line.borrow().chars());
        let common_chars = chars.take_while(|(c1, c2)| c1 == c2);
        let common_byte_len = common_chars.map(|(c, _)| c.len_utf8()).sum();
        comm_pfx = &comm_pfx[0..common_byte_len];
    }
    Some(comm_pfx)
}

/// Get any leading whitespace at the beginning of the given &str (what would be removed by `.trim_start()`)
pub fn leading_whitespace<'a>(s: impl Borrow<&'a str>) -> &'a str {
    let s = s.borrow();
    match s
        .char_indices()
        .take_while(|(_i, c)| c.is_whitespace())
        .last()
    {
        Some((last_ws_idx, _)) => &s[..last_ws_idx + 1],
        None => "",
    }
}

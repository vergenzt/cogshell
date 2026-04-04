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

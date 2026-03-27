use std::iter::zip;

#[cfg(test)]
mod mod_test;

/// Length of the longest common (fully UTF-8) prefix of characters satisfying `char_filter` from
/// the given lines, in bytes. If there is not more than one line, result is empty string.
pub fn common_prefix_of_chars<'a, Iter, F>(lines: Iter, char_filter: F) -> &'a str
where
    Iter: IntoIterator<Item: Into<&'a str>>,
    F: Fn(char) -> bool,
{
    let valid_pfxs = lines.into_iter().map(|line| {
        let line = line.into();
        match line.split_once(|c| !char_filter(c)) {
            Some((pfx, _)) => pfx,
            None => line,
        }
    });

    valid_pfxs
        .reduce(|comm_pfx, curr_pfx| {
            let chars = zip(comm_pfx.chars(), curr_pfx.chars());
            let common_chars = chars.take_while(|(c1, c2)| c1 == c2);
            let common_byte_len = common_chars.map(|(c, _)| c.len_utf8()).sum();
            &comm_pfx[0..common_byte_len]
        })
        .unwrap_or("")
}

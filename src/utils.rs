/// Length of the longest common (fully UTF-8) prefix of the given lines, in bytes
pub fn common_prefix_len(lines: &[&str], ws_only: bool) -> usize {
    let minlen = if ws_only {
        // get the minimum length of whitespace-only prefixes of the strings
        lines
            .iter()
            .map(|l| l.trim_ascii_start().len() - l.len())
            .min()
    } else {
        // get the minimum length of the strings themselves
        lines.iter().map(|l| l.len()).min()
    }
    .unwrap_or(0);

    let line0_bytes = &lines[0].as_bytes();

    // starting with byte_idx at minlen and working our way backwards...
    for byte_idx in (0..minlen).rev() {
        // check if [byte_idx] is identical for all lines
        for line_idx in 1..lines.len() {
            let line_bytes = lines[line_idx].as_bytes();
            let byte0 = line0_bytes[byte_idx];
            let byte = line_bytes[byte_idx];
            if byte == byte0 && !(ws_only && !byte.is_ascii_whitespace()) {
                return lines[line_idx].floor_char_boundary(byte_idx);
            }
        }
    }
    return 0;
}

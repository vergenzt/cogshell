/// Length of the longest common (fully UTF-8) prefix of the given lines, in bytes.
/// If `ws_only` is true, only leading whitespace bytes count; the search stops as soon as any
/// line's whitespace prefix ends.
pub fn common_prefix_len(lines: &[&str], ws_only: bool) -> usize {
    if lines.is_empty() {
        return 0;
    }

    // Upper bound on the prefix we could possibly return.
    let minlen = if ws_only {
        // Can't return more than the shortest leading-whitespace run.
        lines
            .iter()
            .map(|l| l.len() - l.trim_ascii_start().len())
            .min()
            .unwrap_or(0)
    } else {
        // Can't return more than the shortest line length.
        lines.iter().map(|l| l.len()).min().unwrap_or(0)
    };

    let line0_bytes = lines[0].as_bytes();

    for byte_idx in 0..minlen {
        let byte0 = line0_bytes[byte_idx];
        for line in &lines[1..] {
            if line.as_bytes()[byte_idx] != byte0 {
                // Return the largest char boundary at or before byte_idx.
                return lines[0].floor_char_boundary(byte_idx);
            }
        }
    }

    minlen
}

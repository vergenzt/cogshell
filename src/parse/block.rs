use crate::{source::{SourceLocation, SourceMarker}, utils::common_prefix_of_chars};


/// Everything needed to execute an embedded code block
pub struct ParsedBlock<'a> {
    /// The (pre-trimmed) lines of the program to run
    program_lines: &'a [&'a str],
    /// The location from the source file where this block begins
    program_start: SourceLocation,
    /// The text to prepend to lines of output
    prefix_by: &'a str,
    /// The text to append to lines of output
    suffix_by: &'a str,
}

impl ParsedBlock<'_> {
    /// Parse a CogShell block from matched markers
    pub fn new<'a>(content: &'a str, markers: [SourceMarker; 3]) -> ParsedBlock<'a> {
        let [prog_beg, prog_end, outp_end] = markers;

        // save whitespace prefix of the marker lines for prepending to output
        // https://github.com/nedbat/cog/blob/05842d65800458b1a18eba89770d8cb705cb503a/cogapp/cogapp.py#L57-L58
        let prog_marker_ws_pfx = common_prefix_of_chars(

        match (prog_start.preceding_line_starts[..], prog_end.preceding_line_starts[..]) {
            ([line_start], []) => {
                let ws_pfx = common_prefix_of_chars(lines, char_filter)
                let program_line = &content[(*prog_beg.end + 1)..*prog_end.start];
                let (ws_pfx, program_ltrimmed) = program_line_raw.split_at(program_line_raw.find(|c| !c.is_whitespace()).unwrap_or(0));
                let program_line = program_ltrimmed.trim_end();
                let program_start = SourceLocation {
                    line: prog_beg.end.line,
                    col: prog_beg.end.col + ws_pfx.len(),
                    offset: prog_beg.end.offset + ws_pfx.len(),
                };

                Self {
                    program_lines: &[program_line],
                    program_start,
                    prefix_by: 
                }
        }

        // save whitespace prefix of the marker lines for prepending to output
        // https://github.com/nedbat/cog/blob/05842d65800458b1a18eba89770d8cb705cb503a/cogapp/cogapp.py#L57-L58
        let prog_marker_ws_pfx = common_prefix_of_chars(
            if prog_end.preceding_line_starts.is_empty() {
                [][..]
            } else {
                [
                    &content[*prog_beg.line_start..*prog_beg.start],
                    &content[*prog_end.line_start..*prog_end.start],
                ][..]
            },
            |c| c.is_ascii_whitespace(),
        );

        if prog_beg.line_start == prog_end.line_start {
            let prog_start_line = prog_beg.line_start.line;
        } else {
            let prog_lines: Vec<_> = {
                // "full" = with prog_beg expanded to the beginning of its line
                let prog_raw_full = content[*prog_beg.line_start..*prog_end.start];
                let prog_raw_full_lines = prog_raw_full.split_inclusive('\n');

                // from cog implementation: "If the markers and lines all have the same prefix (end-of-line
                // comment chars, for example), then remove it from all the lines."
                let prog_raw_full_common_pfx =
                    common_prefix_of_chars(prog_raw_full_lines, |_| true);
                let prog_clean_lines = prog_raw_full_lines.map(|raw_line| {
                    raw_line
                        .get(prog_raw_full_common_pfx.len()..)
                        .unwrap_or_else(|| {
                            assert!(raw_line.is_empty());
                            &raw_line
                        })
                });

                // dedent program lines

                prog_clean_lines.collect()
            };

            prog_dedented_lines
                .map(|ded_line| &ded_line[..prog_dedented_common_pfx.len()])
                .collect()
        };
    }
}

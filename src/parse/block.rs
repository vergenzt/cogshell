use common_prefix::{common_prefix_of_chars, leading_whitespace};

use super::{checksum::*, marker_inst::*, span::*};

#[derive(Debug)]
pub struct BlockMarkers<'i> {
    pub prog_start: MarkerInst<'i>,
    pub prog_end: MarkerInst<'i>,
    pub outp_end: MarkerInst<'i>,
}

impl<'i> BlockMarkers<'i> {
    pub fn new(markers: &[MarkerInst<'i>; 3]) -> BlockMarkers<'i> {
        let [prog_start, prog_end, outp_end] = *markers;
        Self {
            prog_start,
            prog_end,
            outp_end,
        }
    }
}

/// Everything needed to execute an embedded code block
#[derive(Debug)]
pub struct Block<'i> {
    /// The markers which delimit this block
    pub markers: BlockMarkers<'i>,
    /// The (pre-trimmed) lines of the program to run
    pub prog_lines: Vec<&'i str>,
    /// The text to prepend to lines of output
    pub prog_whitespace_pfx: &'i str,
    /// The unmodified previous output bytes found between the program end and output end markers
    pub output_prev: &'i str,
    /// The previous output checksum which followed this block's output end marker, if present
    pub output_prev_hash: Option<Checksum<'i>>,
    /// The full span of (the parsed version of) this block from start to end
    pub span: Span,
}

impl<'strs> Block<'strs> {
    /// Parse a CogShell block from matched markers
    pub fn new(content: &'strs str, markers: BlockMarkers<'strs>) -> Self {
        let BlockMarkers {
            prog_start,
            prog_end,
            outp_end,
        } = markers;

        // find beginning of line containing start marker
        let prog_content_pfx = &content[..*prog_start.span.start];
        let prog_line1_start_idx = prog_content_pfx.rfind('\n').map(|i| i + 1).unwrap_or(0);

        let mut prog_full_lines: Vec<_> = content[prog_line1_start_idx..*prog_end.span.start]
            .split('\n')
            .collect();

        // save whitespace prefix of the marker lines for prepending to output
        // https://github.com/nedbat/cog/blob/05842d65800458b1a18eba89770d8cb705cb503a/cogapp/cogapp.py#L57-L58
        let prog_whitespace_pfx = {
            let start_ws =
                leading_whitespace(&content[prog_line1_start_idx..*prog_start.span.start]);
            if let Some(&prog_end_line_pfx) = prog_full_lines[1..].last() {
                let end_ws = leading_whitespace(&prog_end_line_pfx);
                common_prefix_of_chars(&vec![start_ws, end_ws]).unwrap_or("")
            } else {
                start_ws
            }
        };

        // from cog implementation: "If the markers and lines all have the same prefix (end-of-line comment chars, for
        // example), then remove it from all the lines."
        // https://github.com/nedbat/cog/blob/05842d65800458b1a18eba89770d8cb705cb503a/cogapp/cogapp.py#L46-L48
        if let Some(prog_pfx_to_strip) = common_prefix_of_chars(&prog_full_lines) {
            for line in prog_full_lines.iter_mut() {
                *line = line.strip_prefix(prog_pfx_to_strip).unwrap();
            }
        }

        // remove pfx and start marker from first line
        let prog_line1_end_idx = prog_line1_start_idx + prog_full_lines[0].len();
        prog_full_lines[0] = &content[(*prog_start.span.end + 1)..prog_line1_end_idx];

        // dedent program lines after the first
        let lines_to_dedent = prog_full_lines[1..].iter().filter(|l| !l.is_empty());
        let line_indents: Vec<_> = lines_to_dedent.map(|l| leading_whitespace(l)).collect();
        if let Some(indent) = common_prefix_of_chars(&line_indents) {
            for line in prog_full_lines[1..].iter_mut() {
                *line = line.strip_prefix(indent).unwrap_or(line);
            }
        }

        let output_prev = content[*prog_end.span.start..*outp_end.span.end]
            .trim_prefix("\n")
            .trim_suffix("\n");
        let block_sfx = &content[*outp_end.span.end..];
        let output_prev_hash = Checksum::from_block_suffix(block_sfx);

        let span = Span {
            start: prog_start.span.start,
            end: outp_end.span.end,
        };

        Self {
            prog_lines: prog_full_lines,
            prog_whitespace_pfx,
            markers,
            output_prev,
            output_prev_hash,
            span,
        }
    }
}

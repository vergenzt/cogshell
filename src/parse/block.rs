extern crate proc_macro;

use crate::{
    parse::{Checksum, FileContext, MarkerInst, Span},
    utils::{common_prefix_of_chars, leading_whitespace},
};

pub struct BlockMarkers<'a> {
    pub prog_beg: MarkerInst<'a>,
    pub prog_end: MarkerInst<'a>,
    pub outp_end: MarkerInst<'a>,
}

impl<'a> BlockMarkers<'a> {
    pub fn new(markers: &[MarkerInst<'a>; 3]) -> BlockMarkers<'a> {
        let [prog_beg, prog_end, outp_end] = *markers;
        Self {
            prog_beg,
            prog_end,
            outp_end,
        }
    }
}

/// Everything needed to execute an embedded code block
pub struct Block<'a> {
    /// The markers which delimit this block
    pub markers: BlockMarkers<'a>,
    /// The (pre-trimmed) lines of the program to run
    pub prog_lines: Vec<&'a [u8]>,
    /// The text to prepend to lines of output
    pub prog_whitespace_pfx: &'a [u8],
    /// The unmodified previous output bytes found between the program end and output end markers
    pub output_prev: &'a [u8],
    /// The previous output checksum which followed this block's output end marker, if present
    pub output_prev_hash: Option<Checksum<'a>>,
    /// The full span of (the parsed version of) this block from start to end
    pub span: Span,
}

impl<'a> Block<'a> {
    /// Parse a CogShell block from matched markers
    pub fn new(ctx: &'a FileContext, markers: BlockMarkers<'a>) -> Self {
        let FileContext { content, .. } = ctx;
        let BlockMarkers {
            prog_beg,
            prog_end,
            outp_end,
        } = &markers;

        // find beginning of line containing start marker
        let prog_pfx = &content[..*prog_beg.span.start];
        let prog_start_line_idx = prog_pfx
            .iter()
            .rposition(|c| *c == b'\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let mut prog_lines: Vec<_> = content[prog_start_line_idx..*prog_end.span.start]
            .split(|b| *b == b'\n')
            .collect();

        // save whitespace prefix of the marker lines for prepending to output
        // https://github.com/nedbat/cog/blob/05842d65800458b1a18eba89770d8cb705cb503a/cogapp/cogapp.py#L57-L58
        let prog_whitespace_pfx = {
            let start_ws = leading_whitespace(&content[prog_start_line_idx..*prog_beg.span.start]);
            if let Some(&prog_end_line_pfx) = prog_lines[1..].last() {
                let end_ws = leading_whitespace(&prog_end_line_pfx);
                common_prefix_of_chars(&vec![start_ws, end_ws]).unwrap()
            } else {
                start_ws
            }
        };

        // from cog implementation: "If the markers and lines all have the same prefix (end-of-line comment chars, for
        // example), then remove it from all the lines."
        // https://github.com/nedbat/cog/blob/05842d65800458b1a18eba89770d8cb705cb503a/cogapp/cogapp.py#L46-L48
        if let Some(prog_pfx_to_strip) = common_prefix_of_chars(&prog_lines) {
            for line in prog_lines.iter_mut() {
                *line = ByteStr::new(line.strip_prefix(prog_pfx_to_strip).unwrap());
            }
        }

        // remove start marker from first line
        prog_lines[0] = ByteStr::new(prog_lines[0][prog_beg.span.len()..].trim_ascii_start());

        // dedent program lines after the first
        let lines_to_dedent = prog_lines[1..].iter().filter(|l| !l.is_empty());
        let line_indents = lines_to_dedent.map(|l| leading_whitespace(l));
        if let Some(indent) = common_prefix_of_chars(line_indents) {
            for line in prog_lines[1..].iter_mut() {
                *line = ByteStr::new(line.strip_prefix(indent).unwrap_or(line));
            }
        }

        let output_prev = content[*prog_end.span.start..*outp_end.span.end]
            .trim_prefix(b"\n")
            .trim_suffix(b"\n");
        let block_sfx = &content[*outp_end.span.end..];
        let output_prev_hash = Checksum::from_block_suffix(block_sfx);

        let span = Span {
            start: prog_beg.span.start,
            end: outp_end.span.end,
        };

        Self {
            prog_lines,
            prog_whitespace_pfx,
            markers,
            output_prev,
            output_prev_hash,
            span,
        }
    }
}

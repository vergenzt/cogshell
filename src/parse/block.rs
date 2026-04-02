extern crate proc_macro;

use std::borrow::Borrow;

use crate::{parse::marker::Marker, utils::common_prefix_of_chars};

/// Everything needed to execute an embedded code block
pub struct ParsedBlock<'a> {
    /// The (pre-trimmed) lines of the program to run
    prog_lines: Vec<&'a str>,
    /// The text to prepend to lines of output
    prog_whitespace_pfx: &'a str,
    /// The unmodified previous output bytes found between the program end and output end markers
    prev_prefixed_output: &'a str,
}

macro_rules! strspan {
    // a -> b = the bytes offset from start of a to start of b
    ($a:ident -> $b:ident) => {{
        assert!($b > $a);
        $b.as_ptr() as usize - $a.as_ptr() as usize
    }};
    // $s[$a..$b] = the &str from the start of $a to start of $b, where $a and $b are substrings of $s
    ( $s:ident [ $($a:ident )? .. $($b:ident)? ]) => {
        & $s [
            $( strspan!($s -> $a) )?  ..
            $( strspan!($s -> $b) )?
        ]
    };
}

fn leading_whitespace<'a>(s: impl Borrow<&'a str>) -> &'a str {
    let s = s.borrow();
    let trimmed = s.trim_start();
    strspan!(s[..trimmed])
}

impl<'a> ParsedBlock<'a> {
    /// Parse a CogShell block from matched markers
    pub fn new(content: &'a str, markers: [Marker; 3]) -> Self {
        let [prog_beg, prog_end, outp_end] = markers.map(|m| m.span.as_str());

        // find beginning of line containing start marker
        let prog_pfx = strspan!(content[..prog_beg]);
        let prog_start_line_pfx = prog_pfx.rsplit('\n').next().unwrap_or(prog_pfx);
        let mut prog_lines: Vec<_> = strspan!(content[prog_start_line_pfx..prog_end])
            .split('\n')
            .collect();

        // save whitespace prefix of the marker lines for prepending to output
        // https://github.com/nedbat/cog/blob/05842d65800458b1a18eba89770d8cb705cb503a/cogapp/cogapp.py#L57-L58
        let prog_whitespace_pfx = {
            let start_ws = leading_whitespace(&prog_start_line_pfx);
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
                *line = line.strip_prefix(prog_pfx_to_strip).unwrap();
            }
        }

        // remove start marker from first line
        prog_lines[0] = prog_lines[0][prog_beg.len()..].trim_start();

        // dedent program lines after the first
        let lines_to_dedent = prog_lines[1..].iter().filter(|l| !l.is_empty());
        let line_indents = lines_to_dedent.map(|l| leading_whitespace(l));
        if let Some(indent) = common_prefix_of_chars(line_indents) {
            for line in prog_lines[1..].iter_mut() {
                *line = line.strip_prefix(indent).unwrap_or(line);
            }
        }

        let prev_prefixed_output = &strspan!(content[prog_end..outp_end])[prog_end.len()..];

        Self {
            prog_lines: prog_lines,
            prog_whitespace_pfx,
            prev_prefixed_output,
        }
    }
}

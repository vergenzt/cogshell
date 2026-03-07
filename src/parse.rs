use miette::Error;
use regex;
use std::array;
use std::iter;
use std::ops::Range;

use miette::{Diagnostic, NamedSource, bail, miette};
use regex::Regex;
use thiserror::Error;

use crate::config::Config;
use crate::utils::common_prefix_len;

#[derive(Error, Diagnostic, Debug)]
pub enum ParseError<'a> {
    #[error("found {mark:?} with no opening {prog_start:?}")]
    ExpectedProgStart {
        #[source_code]
        src: NamedSource<&'static str>,
        #[label("found")]
        mark: Range<usize>,
        prog_start: &'a str,
    },
    #[error("expected {prog_end:?} following {prog_start:?}, but got {mark:?}")]
    ExpectedProgEnd {
        #[source_code]
        src: NamedSource<&'static str>,
        #[label(primary, "expected program close {prog_end:?}")]
        mark: Range<usize>,
        #[label("after program start here")]
        prog_start: Range<usize>,
        prog_end: &'a str,
    },
    #[error("expected {output_end:?} following {prog_start:?} and {prog_end:?}, but got {mark:?}")]
    ExpectedOutputEnd {
        #[source_code]
        src: NamedSource<&'static str>,
        #[label(primary, "expected output close {output_end:?}")]
        mark: Range<usize>,
        #[label("after program start here")]
        prog_start: Range<usize>,
        #[label("and program end here")]
        prog_end: Range<usize>,
        output_end: &'a str,
    },
}

struct ParsedFile<'a> {
    /// The original content of the file
    content: &'a str,
    /// Config used to parse the file
    config: &'a Config<'a>,
    /// The CogShell code blocks parsed from the file
    blocks: Vec<&'a ParsedBlock<'a>>,
}

impl ParsedFile<'_> {
    fn from_content(content: &str, config: &Config, name: Option<&str>) -> Result<Self, Error> {
        let eof = Range { start: content.len(), end: content.len() };
        let marker_re = Regex::new(config.markers.map(regex::escape).join("|").as_str())
            .map_err(|e| miette!(e.to_string()))?;

        let markers_found = marker_re.find_iter(content).collect::<Vec<_>>();
        let (full_triples, leftovers) = markers_found.as_chunks::<3>();
        let triples = full_triples.iter().chain(iter::once(&array::from_fn(|i| {
            // use EOF as a placeholder if there are leftover matches
            *leftovers.get(i).unwrap_or(&eof)
        })));

        let src = NamedSource::new(path.display().to_string(), content);
        let [prog_start, prog_end, output_end] = config.markers;

        let blocks = triples
            .map(|triple| match triple {
                [a, ..] if a.as_str() != prog_start => bail!(ParseError::ExpectedProgStart {
                    src,
                    mark: a.range(),
                    prog_start
                }),
                [a, b, ..] if b.as_str() != prog_end => bail!(ParseError::ExpectedProgEnd {
                    src,
                    mark: b.range(),
                    prog_start: a.range(),
                    prog_end,
                }),
                [a, b, c] if c.as_str() != output_end => bail!(ParseError::ExpectedOutputEnd {
                    src,
                    mark: c.range(),
                    prog_start: a.range(),
                    prog_end: b.range(),
                    output_end
                }),
                [a, b, c] => Ok(ParsedBlock::from(
                    &content,
                    &[a.range(), b.range(), c.range()],
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            path,
            content,
            config,
            blocks,
        })
    }
}

/// Everything needed to execute an embedded code block
struct ParsedBlock<'a> {
    /// The (pre-trimmed) lines of the program to run
    program: &'a [&'a str],
    /// The common whitespace to be prepended to the program's output
    program_common_whitespace: &'a str,
}

impl ParsedBlock<'_> {
    /// Parse a CogShell block from Matches to its markers
    fn from<'a>(content: &'a str, markers: &'a [Range<usize>; 3]) -> ParsedBlock<'_> {
        let [prog_start, prog_end, output_end] = markers;

        let overall_start = content[..prog_start.start()]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let overall_raw = &content[overall_start..output_end.start()];

        let program_raw = &content[(prog_start.end() + 1)..prog_end.start()];
        let (program, output_prev) = if program_raw.contains('\n') {
            let overall_lines = &overall_raw.lines().collect::<Vec<_>>();

            // determine prefix to strip from *all* lines (program & output)
            let pfx_len_for_program_and_output = common_prefix_len(overall_lines, false);
            &[program_raw.trim()]
        } else {
            program_raw
                .lines()
                .map(|line| line.split_at(pfx_len_for_program).1)
                .collect::<Vec<_>>()
                .as_slice()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 1: prog_start and prog_end on the same line — trim whitespace and return
    #[test]
    fn test_rule1_same_line() {
        const CONTENT: &str = "[[[cogsh   echo foo  ]]]\n[[[end]]]";
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(ParsedBlock::extract_program(&block), Ok(vec!["echo foo"]));
    }

    // Rule 2: all lines share a common non-whitespace prefix — strip it, then strip common indent
    #[test]
    fn test_rule2_shared_prefix() {
        const CONTENT: &str = concat!(
            "prologue\n",
            "--[[[cogsh\n",
            "--   echo foo bar baz\n",
            "--   jq -n '1 + 2 + 3'\n",
            "--]]]\n",
            "xy0\n",
            "xy1\n",
            "xy2\n",
            "--[[[end]]]\n",
        );
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(
            ParsedBlock::from(&block),
            Ok(vec!["echo foo bar baz", "jq -n '1 + 2 + 3'"])
        );
    }

    // Rule 3: non-whitespace after prog_start on same line becomes first line (trimmed),
    // remaining lines get common indent stripped (rule 4)
    #[test]
    fn test_rule3_inline_first_line() {
        const CONTENT: &str = concat!(
            "[[[cogsh   first line\n",
            "    second line\n",
            "    third line\n",
            "]]]\n",
            "[[[end]]]",
        );
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(
            ParsedBlock::extract_program(&block),
            Ok(vec!["first line", "second line", "third line"])
        );
    }

    // Rule 4: multiline, no shared non-ws prefix — strip common leading whitespace
    #[test]
    fn test_rule4_common_indent() {
        const CONTENT: &str = concat!(
            "[[[cogsh\n",
            "    echo foo\n",
            "    echo bar\n",
            "]]]\n",
            "[[[end]]]",
        );
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(
            ParsedBlock::extract_program(&block),
            Ok(vec!["echo foo", "echo bar"])
        );
    }

    // Error: prog_end and output_end on the same line
    #[test]
    fn test_error_prog_end_output_end_same_line() {
        const CONTENT: &str = "[[[cogsh echo foo]]] [[[end]]]";
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert!(ParsedBlock::extract_program(&block).is_err());
    }
}

use std::error::Error;
use std::fmt::Display;
use std::ops::Range;

use annotate_snippets::Report;
use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet, renderer::DecorStyle};

use regex::{Match, Regex, escape};

use crate::config::{Config, MarkerConfig};
use crate::utils::common_prefix_len;

#[derive(Debug, Default, PartialEq)]
struct SourceLocation {
    line: usize,
    col: usize,
    offset: usize,
}

impl SourceLocation {
    fn from(offset: usize, line_start: SourceLocation) -> Self {
        SourceLocation {
            line: line_start.line,
            col: offset - line_start.offset,
            offset,
        }
    }
}

#[derive(Debug)]
struct SourceSpan<'a> {
    content: &'a str,
    start: SourceLocation,
    end: SourceLocation,
}

impl PartialEq for SourceSpan<'_> {
    /// Can be equal
    fn eq(&self, other: &Self) -> bool {
        let [self_addrs, other_addrs] = [self, other].map(|span| {
            let addr = span.content.as_ptr().addr();
            (addr + self.start.offset, addr + self.end.offset)
        });
        self_addrs == other_addrs
    }
}

impl<'a> SourceSpan<'a> {
    fn from_range(content: &'a str, found: Range<usize>, line_start: &mut SourceLocation) -> Self {
        let start = SourceLocation::from(found.start, SourceLocation { ..*line_start });

        // increment line to end of the match
        let lines: Vec<_> = content[found.clone()].split_inclusive('\n').collect();
        line_start.line += lines.len() - 1;
        line_start.offset = found.end - lines.last().unwrap().len();

        let end = SourceLocation::from(found.end, SourceLocation { ..*line_start });

        SourceSpan {
            content,
            start,
            end,
        }
    }
}

impl Display for SourceSpan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.content[self.start.offset..self.end.offset])
    }
}

impl From<&SourceSpan<'_>> for Range<usize> {
    fn from(value: &SourceSpan) -> Self {
        value.start.offset..value.end.offset
    }
}

#[cfg(test)]
mod source_span_tests {
    use super::*;

    #[test]
    fn test_source_span_init() {
        let content = "abcd\nefgh\nijkl\n";
        let mut line_start = SourceLocation::default();
        let span = SourceSpan::from_range(content, 1..4, &mut line_start);
        assert_eq!(
            span,
            SourceSpan {
                content,
                start: SourceLocation {
                    line: 0,
                    col: 1,
                    offset: 1,
                },
                end: SourceLocation {
                    line: 0,
                    col: 5,
                    offset: 5,
                }
            },
        )
    }
}

pub struct ParsedFile<'a> {
    /// The name of the file
    pub filename: &'a str,
    /// The original content of the file
    pub content: &'a str,
    /// Config used to parse the file
    pub config: &'a Config<'a>,
    /// The CogShell code blocks parsed from the file
    pub blocks: Vec<ParsedBlock<'a>>,
}

impl<'a> ParsedFile<'_> {
    fn from_content(content: &str, cfg: &Config, filename: &str) -> Result<Self, Report<'a>> {
        // get a regex with matching groups 1-3 being markers, matching group 4 being newline
        let marker_re = cfg.markers.map(|m| escape(&format!("({})", m))).join("\n");
        let marker_or_eol_re =
            Regex::new(&format!("{}|(\n)", marker_re)).expect("shouldn't ever fail");

        let mut parsed_blocks = Vec::new();
        let mut marker_state = Vec::with_capacity(3);
        let mut line_start = SourceLocation::default();

        for found in marker_or_eol_re.captures_iter(content) {
            let found_marker_idx = found
                .iter()
                .skip(1) //
                .enumerate()
                .flat_map(|(i, grp)| grp.and_then(|_| Some(i).filter(|&i| i < cfg.markers.len())))
                .next();
            let found = found.get_match().range();

            match found_marker_idx {
                // just a newline
                None => {
                    line_start.line += 1;
                    line_start.offset = found.end;
                }
                Some(found_marker_idx) => {
                    let marker = SourceSpan::from_range(content, found, &mut line_start);

                    if found_marker_idx == marker_state.len() {
                        marker_state.push(marker)
                    } else {
                        let marker_sought = cfg.markers[marker_state.len()];
                        let source = Snippet::source(content).path(filename);
                        let report = Box::new([Level::ERROR
                            .primary_title(format!(
                                "unexpected marker {marker}, expected {marker_sought}"
                            ))
                            .element(source.clone().annotation(
                                AnnotationKind::Primary.span((&marker).into()).label({
                                    let found_desc = MarkerConfig::LABELS[found_marker_idx];
                                    format!("unexpected {found_desc} marker")
                                }),
                            ))
                            .elements(marker_state.iter().enumerate().map(
                                |(prev_marker_idx, prev_marker)| {
                                    source.clone().annotation(
                                        AnnotationKind::Context.span(prev_marker.into()).label(
                                            {
                                                let prev_desc =
                                                    MarkerConfig::LABELS[prev_marker_idx];
                                                format!("{prev_desc} marker")
                                            },
                                        ),
                                    )
                                },
                            ))]);
                        return Err(report);
                    }
                }
            }

            if marker_state.len() == cfg.markers.len() {
                let marker_locs = marker_state.split_off(0).as_array().unwrap();
                let block = ParsedBlock::from(content, marker_locs);
                parsed_blocks.push(block);
            }
        }

        Ok(Self {
            filename,
            content,
            config: cfg,
            blocks: parsed_blocks,
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
    fn from<'a>(content: &'a str, markers: &'a [SourceSpan; 3]) -> ParsedBlock<'a> {
        let [prog_start, prog_end, output_end] = markers;

        let overall_start = content[..prog_start.start]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let overall_raw = &content[overall_start..output_end.start];

        let program_raw = &content[(prog_start.end + 1)..prog_end.start];
        let program = if program_raw.contains('\n') {
            // determine prefix to strip from *all* lines (program & output)
            let overall_lines = &overall_raw.lines().collect::<Vec<_>>();
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

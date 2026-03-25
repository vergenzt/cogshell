use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use anyhow::Result;

use num_traits::FromPrimitive;
use regex::{Regex, escape};

use crate::config::{Config, MarkerKind};
use crate::source::{SourceLocation, SourceMarker, SourceSpan};
use crate::utils::common_prefix_len;

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

impl<'a> ParsedFile<'a> {
    fn from_content(
        content: &'a str,
        config: &'a Config,
        filename: &'a str,
    ) -> Result<Self, ParseError<'a>> {
        // get a regex with matching groups 1-3 being markers, matching group 4 being newline
        let marker_or_eol_re = {
            let marker_re = config
                .markers
                .map(|m| escape(&format!("({})", m)))
                .join("|");
            Regex::new(&format!("{}|(\n)", marker_re)).expect("shouldn't ever fail")
        };

        let mut parsed_blocks: Vec<ParsedBlock<'_>> = Vec::new();
        let mut parse_state: Vec<SourceMarker<'_>> = Vec::with_capacity(3);
        let mut line_start = SourceLocation::default();

        for found in marker_or_eol_re.captures_iter(content) {
            let found_idx_opt = found
                .iter()
                .skip(1) //
                .enumerate()
                .flat_map(|(i, grp)| {
                    grp.and_then(|_| Some(i).filter(|&i| i < config.markers.len()))
                })
                .next();
            let found_range = found.get_match().range();

            match found_idx_opt {
                // just a newline -> increment counters
                None => {
                    line_start.line += 1;
                    line_start.offset = found_range.end;
                }
                // a marker
                Some(found_idx) => {
                    let kind = MarkerKind::from_usize(found_idx).unwrap();
                    let span = SourceSpan::from_range(content, found_range, &mut line_start);
                    let marker = SourceMarker { kind, span };
                    if found_idx == parse_state.len() {
                        parse_state.push(marker)
                    } else {
                        return Err(ParseError {
                            config,
                            filename,
                            content,
                            marker_found: Some(marker),
                            parse_state,
                        });
                    }
                }
            }

            if parse_state.len() == config.markers.len() {
                let marker_locs: [SourceMarker; 3] = parse_state
                    .split_off(0)
                    .try_into()
                    .expect("parse_state should have length 3");
                let block = ParsedBlock::from(content, marker_locs, &mut line_start);
                parsed_blocks.push(block);
            }
        }

        if !parse_state.is_empty() {
            return Err(ParseError {
                config,
                filename,
                content,
                marker_found: None,
                parse_state,
            });
        }

        Ok(Self {
            filename,
            content,
            config,
            blocks: parsed_blocks,
        })
    }
}

/// Everything needed to execute an embedded code block
struct ParsedBlock<'a> {
    /// The (pre-trimmed) lines of the program to run
    program_lines: &'a [&'a str],
    /// The text to prepend to lines of output
    prefix_by: &'a str,
    /// The text to append to lines of output
    suffix_by: &'a str,
}

impl ParsedBlock<'_> {
    /// Parse a CogShell block from Matches to its markers
    fn from<'a>(
        content: &'a str,
        markers: [SourceMarker; 3],
        line_start: &SourceLocation,
    ) -> ParsedBlock<'a> {
        let [prog_start, prog_end, outp_end] = markers;

        let prefix_in = {
            let lines: Vec<_> = content[(*prog_start.line_start + 1)..*prog_end.start].lines().collect();
            lines[0][..common_prefix_len(&lines, false)]
        };

        let [prog_line1, prog_tail @ ..] = 
            .collect::<Vec<_>>()[..];

        // leading chars in common to program lines are prefixed to program output
        let arbitrary_pfx = {
            let prog_lines_raw: Vec<_> = content[*prog_start.line_start..*prog_end.end]
                .lines()
                .collect();
            let pfx_len = common_prefix_len(&prog_lines_raw, false);
            &prog_lines_raw[0][..pfx_len]
        };

        let whitespace_prefix = common_prefix_len(&prog_lines_raw, true);

        let program_lines = {
            program_raw
                .lines()
                .map(|line| line.split_at(whitespace_prefix).1)
                .collect::<Vec<_>>()
                .as_slice()
        };
    }
}

#[derive(Debug)]
pub struct ParseError<'a> {
    pub config: &'a Config<'a>,
    pub filename: &'a str,
    pub content: &'a str,
    pub marker_found: Option<SourceMarker<'a>>,
    pub parse_state: Vec<SourceMarker<'a>>,
}

impl ParseError<'_> {
    fn print(&self) {
        let marker_sought: &str = &self.config.markers[self.parse_state.len()];
        let marker_sought_desc: &str = MarkerKind::from_usize(self.parse_state.len())
            .unwrap()
            .into();

        let source = Snippet::source(self.content).path(self.filename);
        let state_elements =
            self.parse_state
                .iter()
                .enumerate()
                .map(|(prev_marker_idx, prev_marker)| {
                    source.annotation(
                        AnnotationKind::Context
                            .span(prev_marker.into())
                            .label(prev_marker.kind.into()),
                    )
                });
        let level = Level::ERROR;

        let error = if let Some(marker_found) = &self.marker_found {
            let marker_found_kind = marker_found.kind;
            level.primary_title(format!(
                "unexpected {marker_found_kind} marker {marker_found}, expected {marker_sought_desc} marker {marker_sought}"
            ))
            .element(
                source.clone().annotation(
                    AnnotationKind::Primary
                        .span(marker_found.into())
                        .label(format!("unexpected {marker_found_kind} marker")),
                )
            )
        } else {
            level.primary_title(format!(
                "unexpected end of file, expected {marker_sought_desc} marker {marker_sought}"
            ))
        };

        let report = Renderer::styled().render(&[error]);
        anstream::eprintln!("{}", report);
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

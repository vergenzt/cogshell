use std::iter::chain;
use std::ops::Range;

use anyhow::Result;

use regex::Regex;

use crate::config::Config;
use crate::parse::block::ParsedBlock;
use crate::parse::error::{ParseError, ParseErrorContext, ParseErrorKind};
use crate::source::{MarkerKind, SourceLocation, SourceMarker, SourceSpan};

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
        // regex with groups 1-3 being the configured markers, 4 being newline, and 5 being EOF
        let marker_or_eol = {
            let groups = chain(
                config.markers.map(regex::escape),
                [r"\n", r"\z"].map(String::from),
            );
            let groups_vec: Vec<_> = groups.map(|pat| format!("({})", pat)).collect();
            let groups_str = groups_vec.join("|");
            Regex::new(&groups_str).unwrap()
        };

        let mut parsed_blocks: Vec<ParsedBlock<'_>> = Vec::new();
        let mut parse_state: Vec<SourceMarker<'_>> = Vec::with_capacity(3);
        let mut parse_state_line_starts: Vec<SourceLocation> = Vec::new();
        let mut line_start = SourceLocation::default();
        let mut eof: Option<usize> = None;

        for found_caps in marker_or_eol.captures_iter(content) {
            let found = found_caps.get_match();
            let found_range = found.range();
            let found_text = found.as_str();
            let found_grps = found_caps.iter().enumerate();

            let found_marker = found_grps
                .flat_map(|(i, _)| MarkerKind::ALL.get(i - 1))
                .map(|&kind| {
                    let span =
                        SourceSpan::from_range(content, found_range.clone(), &mut line_start);
                    let preceding_line_starts = if parse_state.len() == 0 {
                        assert!(parse_state_line_starts.len() == 0);
                        vec![line_start]
                    } else {
                        parse_state_line_starts.split_off(0)
                    };
                    SourceMarker {
                        kind,
                        span,
                        preceding_line_starts,
                    }
                })
                .next();

            match found_marker {
                // just a newline
                None if found_text == "\n" => {
                    // increment counters
                    line_start.line += 1;
                    line_start.offset = found_range.end;
                    // save line start if we're actively parsing a block
                    if !parse_state.is_empty() {
                        parse_state_line_starts.push(line_start)
                    }
                }
                // correct next expected marker
                Some(marker) if marker.kind as usize == parse_state.len() => {
                    parse_state.push(marker);

                    // check for complete marker set
                    if parse_state.len() == config.markers.len() {
                        let markers: [SourceMarker; 3] =
                            parse_state.split_off(0).try_into().unwrap();
                        let block = ParsedBlock::new(content, markers);
                        parsed_blocks.push(block);
                    }
                }
                // unexpected marker!
                Some(marker) => {
                    return Err(ParseError {
                        kind: ParseErrorKind::UnexpectedMarker(marker),
                        ctx: ParseErrorContext {
                            config,
                            filename,
                            content,
                            parse_state,
                        },
                    });
                }
                // EOF
                _ => {
                    assert_eq!(found_range.start, found_range.end);
                    eof = Some(found_range.start);
                }
            }
        }

        match parse_state[..] {
            // all markers matched
            [] => Ok(Self {
                filename,
                content,
                config,
                blocks: parsed_blocks,
            }),

            // leftover unmatched markers
            [..] => Err(ParseError {
                kind: ParseErrorKind::UnexpectedEOF(eof.unwrap()),
                ctx: ParseErrorContext {
                    config,
                    filename,
                    content,
                    parse_state,
                },
            }),
        }
    }
}

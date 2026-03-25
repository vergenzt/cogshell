use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use anyhow::Result;

use num_traits::FromPrimitive;
use regex::{Regex, escape};

use crate::config::{Config, MarkerKind};
use crate::parse::block::ParsedBlock;
use crate::parse::error::ParseError;
use crate::source::{SourceLocation, SourceMarker, SourceSpan};
use crate::utils::common_prefix_of_chars;

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
        let mut parse_state_line_starts: Vec<SourceLocation> = Vec::new();
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
                // just a newline
                None => {
                    // increment counters
                    line_start.line += 1;
                    line_start.offset = found_range.end;
                    // save line start if we're actively parsing a block
                    if !parse_state.is_empty() {
                        parse_state_line_starts.push(line_start)
                    }
                }
                // a marker
                Some(found_idx) => {
                    let kind = MarkerKind::from_usize(found_idx).unwrap();
                    let span = SourceSpan::from_range(content, found_range, &mut line_start);
                    let preceding_line_starts = if parse_state.len() == 0 {
                        assert!(parse_state_line_starts.len() == 0);
                        vec![line_start]
                    } else {
                        parse_state_line_starts.split_off(0)
                    };

                    let marker = SourceMarker {
                        kind,
                        span,
                        preceding_line_starts,
                    };
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
                let markers: [SourceMarker; 3] = parse_state.split_off(0).try_into().unwrap();
                let block = ParsedBlock::new(content, markers);
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

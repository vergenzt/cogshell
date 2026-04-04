use regex::{Match, Regex};

use super::block::ParsedBlock;
use super::marker::MarkerKind;
use crate::config::Config;
use crate::errors::{ParseError, ParseErrorKind};

pub struct ParseContext<'a> {
    pub config: &'a Config<'a>,
    pub filename: &'a str,
    pub content: &'a str,
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

impl<'a> ParsedFile<'a> {
    fn from_content(
        content: &'a str,
        config: &'a Config,
        filename: &'a str,
    ) -> Result<Self, ParseError<'a>> {
        let markers_re = {
            let pats = config.marker_strings.map(regex::escape);
            let grps = pats.map(|pat| format!("({})", pat));
            Regex::new(&grps.join("|")).unwrap()
        };

        let mut blocks: Vec<ParsedBlock> = Vec::new();
        let mut state: Vec<Match> = Vec::with_capacity(3);

        for caps in markers_re.captures_iter(content) {
            let marker = caps.get_match();
            let marker_kind: MarkerKind = {
                let grp_idx = (1..=3).find_map(|i| caps.get(i).and(Some(i))).unwrap();
                (grp_idx - 1).into()
            };

            if marker_kind as usize == state.len() {
                state.push(marker);

                // check for complete marker set
                if state.len() == 3 {
                    let markers: [Match; 3] = state.split_off(0).try_into().unwrap();
                    let block = ParsedBlock::new(content, markers);
                    blocks.push(block);
                }
            } else {
                return Err(ParseError {
                    kind: ParseErrorKind::UnexpectedMarker(marker_kind, marker),
                    state,
                    ctx: ParseContext {
                        config,
                        filename,
                        content,
                    },
                });
            }
        }

        // unmatched markers left over
        if !state.is_empty() {
            return Err(ParseError {
                kind: ParseErrorKind::UnexpectedEOF,
                state,
                ctx: ParseContext {
                    config,
                    filename,
                    content,
                },
            });
        }

        // all markers matched
        Ok(Self {
            filename,
            content,
            config,
            blocks,
        })
    }
}

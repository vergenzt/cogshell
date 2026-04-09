use std::collections::VecDeque;
use std::fmt::Display;
use std::{array, fs, io};

use regex::{Match, Regex};

use super::block::ParsedBlock;
use super::marker::MarkerKind;
use crate::config::Config;
use crate::errors::{ParseError, ParseErrorKind};
use crate::parse::Marker;

pub struct FileParser<'a> {
    /// The name of the file
    pub filename: &'a str,
    /// The original content of the file
    pub content: String,
    /// Config used to parse the file
    pub config: &'a Config<'a>,
}

pub struct ParsedFile<'a> {
    /// The context used to parse the file
    pub ctx: &'a FileParser<'a>,
    /// The CogShell code blocks parsed from the file
    pub blocks: Vec<ParsedBlock<'a>>,
}

impl<'a> FileParser<'a> {
    pub fn from_file(filename: &'a str, config: &'a Config) -> io::Result<Self> {
        let content = fs::read_to_string(filename)?;
        Ok(Self {
            filename,
            content,
            config,
        })
    }

    pub fn parse(&'a self) -> Result<ParsedFile, ParseError<'a>> {
        let Self {
            content, config, ..
        } = self;

        let markers_re = {
            let pats = config.marker_strings.map(regex::escape);
            let grps = pats.map(|pat| format!("({})", pat));
            let grps_or_nl = format!(r"{}|\n", grps.join("|"));
            Regex::new(&grps_or_nl).unwrap()
        };

        let mut line: usize = 0;
        let mut line_start: usize = 0;
        let mut blocks: Vec<ParsedBlock> = Vec::new();
        let mut state: VecDeque<Match> = VecDeque::with_capacity(3);

        for caps in markers_re.captures_iter(content) {
            if caps.get_match().as_str() == "\n" {
                line += 1;
                line_start = caps.get_match().end();
                continue;
            }

            let marker = caps.get_match();
            let marker_kind: MarkerKind = {
                let grp_idx = (1..=3).find_map(|i| caps.get(i).and(Some(i))).unwrap();
                (grp_idx - 1).into()
            };

            if marker_kind as usize == state.len() {
                state.push_back(marker);

                // check for complete marker set
                if state.len() == 3 {
                    let span = state.pop_front().unwrap();
                    let markers = array::from_fn(|_| Marker {
                        span,
                        line,
                        col: span.start() - line_start,
                    });
                    let block = ParsedBlock::new(self, markers);
                    blocks.push(block);
                }
            } else {
                return Err(ParseError {
                    kind: ParseErrorKind::UnexpectedMarker(marker_kind, marker),
                    state,
                    ctx: self,
                });
            }
        }

        // unmatched markers left over
        if !state.is_empty() {
            return Err(ParseError {
                kind: ParseErrorKind::UnexpectedEOF,
                state,
                ctx: self,
            });
        }

        // all markers matched
        Ok(ParsedFile { ctx: self, blocks })
    }
}

use std::collections::VecDeque;
use std::ops::{Deref, Range};
use std::{array, fs, io};

use regex::{Match, Regex};

use super::block::Block;
use super::errors::{ParseError, ParseErrorKind};
use super::marker::MarkerKind;
use crate::args::io::{FileOrStream, In};
use crate::config::Config;
use crate::parse::{BlockMarkers, Loc, MarkerInst, Span};

pub struct FileContext<'a> {
    /// The filename or input stream containing CogShell block(s)
    pub source: FileOrStream<In>,
    /// The original content of the source
    pub content: String,
    /// Config used to parse the source
    pub config: &'a Config,
}

impl<'a> FileContext<'a> {
    pub fn new(input: FileOrStream<In>, config: &'a Config) -> io::Result<Self> {
        let mut content = String::new();
        &input.open()?.read_to_string(&mut content);
        Ok(Self {
            source: input,
            content,
            config,
        })
    }
}

pub struct File<'a> {
    /// The context used to parse the file
    pub ctx: &'a FileContext<'a>,
    /// The CogShell code blocks parsed from the file
    pub blocks: Vec<Block<'a>>,
}

impl<'a> Deref for File<'a> {
    type Target = FileContext<'a>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<'a> File<'a> {
    pub fn from(ctx: &'a FileContext<'a>) -> Result<File<'a>, ParseError<'a>> {
        let content = &ctx.content;

        let markers_re = {
            let pats = ctx.config.markers.each_ref().map(|s| regex::escape(s));
            let grps = pats.map(|pat| format!("({})", pat));
            let grps_or_nl = format!(r"{}|\n", grps.join("|"));
            Regex::new(&grps_or_nl).unwrap()
        };

        let mut line: usize = 0;
        let mut line_start: usize = 0;
        let mut blocks: Vec<Block> = Vec::new();
        let mut state: Vec<MarkerInst> = Vec::with_capacity(3);

        for caps in markers_re.captures_iter(content) {
            let mat = caps.get_match();

            // just a newline -> increment our line count and skip
            if mat.as_str() == "\n" {
                line += 1;
                line_start = mat.end();
                continue;
            }

            // determine marker kind based on
            let kind: MarkerKind = {
                let grp_idx = (1..=3).find_map(|i| caps.get(i).and(Some(i))).unwrap();
                (grp_idx - 1).into()
            };
            let start = Loc {
                offset: mat.range().start,
                line,
                col: mat.range().start - line_start,
            };
            let end = start + mat.as_str();
            let span = Span { start, end };
            let marker = MarkerInst::new(content, span);

            if kind as usize == state.len() {
                state.push(marker);

                // check for complete marker set
                if state.len() == 3 {
                    let markers = BlockMarkers::new(state.split_off(0).as_array().unwrap());
                    let block = Block::new(ctx, markers);
                    blocks.push(block);
                }
            } else {
                return Err(ParseError {
                    kind: ParseErrorKind::UnexpectedMarker(kind, marker),
                    state,
                    ctx,
                });
            }
        }

        // unmatched markers left over
        if !state.is_empty() {
            return Err(ParseError {
                kind: ParseErrorKind::UnexpectedEOF,
                state,
                ctx,
            });
        }

        // all markers matched
        Ok(File { ctx, blocks })
    }
}

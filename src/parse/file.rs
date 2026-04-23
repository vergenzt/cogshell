use std::ops::{Deref, Range};

use std::{array, fs, io, iter};

use regex::bytes::{Match, Regex};

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
    pub content: Vec<u8>,
    /// Config used to parse the source
    pub config: &'a Config,
}

impl<'a> FileContext<'a> {
    pub fn new(source: FileOrStream<In>, config: &'a Config) -> io::Result<Self> {
        let mut content = vec![];
        source.open()?.read_to_end(&mut content)?;
        Ok(Self {
            source,
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

        let markers_re = Regex::new({
            let re_buf = String::from("(?-u)");
            for (i, marker) in ctx.config.markers.iter().enumerate() {
                if i > 0 {
                    re_buf.push('|');
                }
                re_buf.push('(');
                for byte in marker.iter() {
                    re_buf.push_str(&format!(r"\x{:02x}", byte));
                }
                re_buf.push(')');
            }
            &re_buf
        })
        .unwrap();

        let mut line: usize = 0;
        let mut line_start: usize = 0;
        let mut blocks: Vec<Block> = Vec::new();
        let mut state: Vec<MarkerInst> = Vec::with_capacity(3);

        for caps in markers_re.captures_iter(content) {
            let mat = caps.get_match();

            // just a newline -> increment our line count and skip
            if mat.as_bytes() == &['\n' as u8] {
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
            let end = start + mat.as_bytes();
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

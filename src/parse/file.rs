use std::ops::Deref;

use std::io;
use std::str::FromStr;

use regex::Regex;

use super::block::Block;
use super::errors::{ParseError, ParseErrorKind};
use super::marker::MarkerKind;
use crate::args::{Args, MarkerDefs, Pipe, SourceAndDestArgs};
use crate::parse::{BlockMarkers, Loc, MarkerInst, Span};

#[derive(Debug)]
pub struct FileContext<'a> {
    /// The filename or input stream containing CogShell block(s)
    pub source: &'a Pipe,
    /// The original content of the source
    pub content: String,
    /// Config used to parse the source
    pub config: &'a Args,
}

impl<'a> FileContext<'a> {
    pub fn new(source: &'a Pipe, config: &'a Args) -> io::Result<Self> {
        let mut content = String::new();
        source.open_for_read()?.read_to_string(&mut content)?;
        Ok(Self {
            source,
            content,
            config,
        })
    }
}

struct BlockBuilder<'a> {
    ctx: &'a FileContext<'a>,
    state: Vec<MarkerInst<'a>>,
    blocks: Vec<Block<'a>>,
}

impl<'a> BlockBuilder<'a> {
    pub fn new(ctx: &'a FileContext<'a>) -> Self {
        Self {
            ctx,
            state: vec![],
            blocks: vec![],
        }
    }

    pub fn push_marker(
        &mut self,
        kind_idx: usize,
        marker: MarkerInst<'a>,
    ) -> Result<(), ParseError<'a>> {
        if kind_idx == self.state.len() {
            self.state.push(marker);
            if self.state.len() == MarkerKind::ALL.len() {
                let markers = BlockMarkers::new(self.state.split_off(0).as_array().unwrap());
                let block = Block::new(self.ctx, markers);
                self.blocks.push(block);
            }
            Ok(())
        } else {
            let kind = MarkerKind::ALL[kind_idx];
            let ekind = ParseErrorKind::UnexpectedMarker(kind, marker);
            let state = self.state.clone();
            let ctx = self.ctx;
            Err(ParseError { ekind, state, ctx })
        }
    }

    pub fn finish(self) -> Result<Vec<Block<'a>>, ParseError<'a>> {
        // unmatched markers left over
        if self.state.is_empty() {
            Ok(self.blocks)
        } else {
            let Self { state, ctx, .. } = self;
            Err(ParseError {
                ekind: ParseErrorKind::UnexpectedEOF,
                state,
                ctx,
            })
        }
    }
}

impl MarkerDefs {}

#[derive(Debug)]
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
            let mut re = String::new();
            re.push_str(r"\n");
            for marker in ctx.config.markers.each_ref() {
                re.push_str("|(");
                re.push_str(&regex::escape(marker));
                re.push_str(")");
            }
            Regex::new(&re).unwrap()
        };

        let mut builder = BlockBuilder::new(ctx);
        let mut line = 0;
        let mut line_start = 0;

        for caps in markers_re.captures_iter(content) {
            let mat = caps.get_match();

            // just a newline -> increment our line count and skip
            if mat.as_str() == "\n" {
                line += 1;
                line_start = mat.end();
                continue;
            }

            // determine marker kind based on which capture group matched
            let kind_idx = (0..3).find(|i| caps.get(i + 1).is_some()).unwrap();
            let marker = MarkerInst::new(content, mat, line, line_start);

            builder.push_marker(kind_idx, marker)?;
        }

        let blocks = builder.finish()?;

        // all markers matched
        Ok(File { ctx, blocks })
    }
}

#[test]
fn test() {
    let content = String::from(
        "\
content before
<!--[[[cogsh echo hello]]]-->
STALE OUTPUT
<!--[[[end]]]-->
content after
",
    );
    let ctx = FileContext {
        source: &Pipe::Stream,
        content,
        config: &Args {
            source_and_dest: SourceAndDestArgs::SingleSourceAndDest(Pipe::Stream, Pipe::Stream),
            prologue: vec![],
            output_line_suffix: String::new(),
            markers: MarkerDefs::from_str("[[[cogsh ]]] [[[end]]]").unwrap(),
        },
    };
    println!("{:?}", File::from(&ctx).unwrap());
}

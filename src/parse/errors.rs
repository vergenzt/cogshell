use std::collections::VecDeque;
use std::fmt::Display;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use regex::Match;

use crate::args::io::FileOrStream;
use crate::args::io::InOrOut;
use crate::parse::FileContext;
use crate::parse::MarkerInst;
use crate::parse::MarkerKind;

#[derive(Debug)]
pub enum ParseErrorKind<'a> {
    UnexpectedMarker(MarkerKind, MarkerInst<'a>),
    UnexpectedEOF,
}

pub struct ParseError<'a> {
    pub kind: ParseErrorKind<'a>,
    pub state: Vec<MarkerInst<'a>>,
    pub ctx: &'a FileContext<'a>,
}

impl<T: InOrOut> Display for FileOrStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl ParseError<'_> {
    fn print(&self) {
        let ParseError { kind, ctx, state } = self;
        let sought_idx = state.len();
        let sought_str = String::from_utf8_lossy(&ctx.config.markers[sought_idx]);
        let sought_kind: MarkerKind = sought_idx.into();

        let source =
            Snippet::source(String::from_utf8_lossy(&ctx.content)).path(ctx.source.to_string());
        let error = Level::ERROR;

        let error = match kind {
            ParseErrorKind::UnexpectedMarker(kind, marker) => {
                let str = String::from_utf8_lossy(marker.bytes());
                error
                    .primary_title(format!(
                        "unexpected {kind} {str}, expected {sought_kind} {sought_str}"
                    ))
                    .element(
                        source.clone().annotation(
                            AnnotationKind::Primary
                                .span(marker.span.into())
                                .label(format!("unexpected {kind}")),
                        ),
                    )
            }
            ParseErrorKind::UnexpectedEOF => {
                let eof = ctx.content.len();
                error
                    .primary_title(format!(
                        "unexpected end of file, expected {sought_kind} {sought_str}"
                    ))
                    .element({
                        source
                            .clone()
                            .annotation(AnnotationKind::Primary.span(eof..eof).label("EOF"))
                    })
            }
        };

        let prev_markers =
            state.iter().enumerate().map(|(i, prev_marker)| {
                source.clone().annotation(
                    AnnotationKind::Context
                        .span(prev_marker.span.into())
                        .label({
                            let kind: MarkerKind = i.into();
                            kind.description()
                        }),
                )
            });
        let error = error.elements(prev_markers);

        let report = Renderer::styled().render(&[error]);
        anstream::eprintln!("{}", report);
    }
}

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use regex::Match;

use crate::parse::MarkerKind;
use crate::parse::ParseContext;

#[derive(Debug)]
pub enum ParseErrorKind<'a> {
    UnexpectedMarker(MarkerKind, Match<'a>),
    UnexpectedEOF,
}

pub struct ParseError<'a> {
    pub kind: ParseErrorKind<'a>,
    pub state: Vec<Match<'a>>,
    pub ctx: ParseContext<'a>,
}

impl ParseError<'_> {
    fn print(&self) {
        let ParseError { kind, ctx, state } = self;
        let sought_idx = state.len();
        let sought_str: &str = &ctx.config.marker_strings[sought_idx];
        let sought_kind: MarkerKind = sought_idx.into();

        let source = Snippet::source(ctx.content).path(ctx.filename);
        let error = Level::ERROR;

        let error = match kind {
            ParseErrorKind::UnexpectedMarker(kind, span) => {
                let str = span.as_str();
                error
                    .primary_title(format!(
                        "unexpected {kind} {str}, expected {sought_kind} {sought_str}"
                    ))
                    .element(
                        source.clone().annotation(
                            AnnotationKind::Primary
                                .span(span.range())
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

        let prev_markers = state.iter().enumerate().map(|(i, prev_marker)| {
            source
                .clone()
                .annotation(AnnotationKind::Context.span(prev_marker.range()).label({
                    let kind: MarkerKind = i.into();
                    kind.description()
                }))
        });
        let error = error.elements(prev_markers);

        let report = Renderer::styled().render(&[error]);
        anstream::eprintln!("{}", report);
    }
}

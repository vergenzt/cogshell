use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};

use crate::config::Config;
use crate::source::{MarkerKind, SourceMarker};

#[derive(Debug)]
pub struct ParseErrorContext<'a> {
    pub config: &'a Config<'a>,
    pub filename: &'a str,
    pub content: &'a str,
    pub parse_state: Vec<SourceMarker<'a>>,
}

#[derive(Debug)]
pub enum ParseErrorKind<'a> {
    UnexpectedMarker(SourceMarker<'a>),
    UnexpectedEOF,
}

pub struct ParseError<'a> {
    pub kind: ParseErrorKind<'a>,
    pub ctx: ParseErrorContext<'a>,
}

impl ParseError<'_> {
    fn print(&self) {
        let ParseError { kind, ctx } = self;
        let sought_idx = ctx.parse_state.len();
        let sought_str: &str = &ctx.config.markers[sought_idx];
        let sought_kind: MarkerKind = MarkerKind::ALL[sought_idx];

        let source = Snippet::source(ctx.content).path(ctx.filename);
        let error = Level::ERROR;

        let error = match kind {
            ParseErrorKind::UnexpectedMarker(SourceMarker { kind, span, .. }) => {
                let found_str = &span.content[*span.start..*span.end];
                error.primary_title(format!(
                    "unexpected {kind} marker {found_str}, expected {sought_kind} marker {sought_str}"
                ))
                .element(
                    source.clone().annotation(
                        AnnotationKind::Primary
                            .span(*span.start..*span.end)
                            .label(format!("unexpected {kind} marker")),
                    )
                )
            }
            ParseErrorKind::UnexpectedEOF => {
                let eof = ctx.content.len();
                error
                    .primary_title(format!(
                        "unexpected end of file, expected {sought_kind} marker {sought_str}"
                    ))
                    .element({
                        source
                            .clone()
                            .annotation(AnnotationKind::Primary.span(*eof..*eof).label("EOF"))
                    })
            }
        };

        let prev_marker_ctx = ctx.parse_state.iter().map(|prev_marker| {
            source.clone().annotation(
                AnnotationKind::Context
                    .span(prev_marker.into())
                    .label(prev_marker.kind.description()),
            )
        });
        let error = error.elements(prev_marker_ctx);

        let report = Renderer::styled().render(&[error]);
        anstream::eprintln!("{}", report);
    }
}

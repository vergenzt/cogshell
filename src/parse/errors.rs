use std::io;

use annotate_snippets::{AnnotationKind, Level, Origin, Renderer};

use super::{marker_inst::*, marker_kind::*, parse_state::*};
use crate::args;

/// Construct an `io::Error` for the given state. If `found_marker_kind` is `Some(...)`,
/// then returns an `io::ErrorKind::InvalidData`. If `None`, then returns an
/// `io::ErrorKind::UnexpectedEof`.
pub fn err_unexpected_marker<'a>(
    state: &ParseFileState<'a>,
    found_marker: Option<(MarkerKind, MarkerInst<'a>)>,
) -> io::Error {
    let sought_idx = state.open_markers.len();
    let sought_str = &args.markers[sought_idx];
    let sought_kind = MarkerKind::ALL[sought_idx];

    // let source = Snippet::source(&state.content).path(state.source.to_string());

    let (ekind, report) = match found_marker {
        Some((kind, marker)) => {
            let origin = {
                let loc = marker.span.start;
                Origin::path(state.source.to_string())
                    .line(loc.line)
                    .char_column(loc.col)
            };
            let str = marker.as_str();

            (
                io::ErrorKind::InvalidData,
                Level::ERROR
                    .primary_title(format!(
                        "unexpected {kind} {str}, expected {sought_kind} {sought_str}"
                    ))
                    .element(
                        source.clone().annotation(
                            AnnotationKind::Primary
                                .span(marker.span.into())
                                .label(format!("unexpected {kind}")),
                        ),
                    ),
            )
        }
        None => {
            let eof = state.content.len();
            (
                io::ErrorKind::UnexpectedEof,
                Level::ERROR
                    .primary_title(format!(
                        "unexpected end of file, expected {sought_kind} {sought_str}"
                    ))
                    .element({
                        source
                            .clone()
                            .annotation(AnnotationKind::Primary.span(eof..eof).label("EOF"))
                    }),
            )
        }
    };

    let prev_markers =
        state
            .open_markers
            .iter()
            .enumerate()
            .map(|(i, prev_marker)| {
                source.clone().annotation(
                    AnnotationKind::Context
                        .span(prev_marker.span.into())
                        .label({
                            let kind = MarkerKind::ALL[i];
                            kind.description()
                        }),
                )
            });
    let report = report.elements(prev_markers);
    let msg = Renderer::plain().render(&[report]);

    io::Error::new(ekind, msg)
}

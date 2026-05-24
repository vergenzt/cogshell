use std::{io, mem};

use annotate_snippets::{Annotation, AnnotationKind, Level, Renderer, Snippet};

use super::{marker_inst::*, marker_kind::*, parse_state::*};

fn prev_marker_annotations<'s, 'i>(
    state: &'s ParseFileState<'_, 'i>,
    snippet: &'s impl Fn() -> Snippet<'i, Annotation<'i>>,
) -> impl Iterator<Item = Snippet<'i, Annotation<'i>>> {
    state
        .open_markers
        .iter()
        .enumerate()
        .map(|(i, prev_marker)| {
            snippet().annotation(
                AnnotationKind::Context
                    .span(prev_marker.span.into())
                    .label(MarkerKind::ALL[i].description()),
            )
        })
}

/// Construct an [`std::io::Error`] describing a marker-sequence problem.
///
///  * `Some((kind, marker))` => `InvalidData` (saw a marker we didn't expect).
///  * `None` => `UnexpectedEof` (hit EOF mid-block).
pub fn err_unexpected_marker<'a, 'i>(
    state: &ParseFileState<'a, 'i>,
    found_marker: Option<MarkerInst<'i>>,
) -> io::Error {
    let args = state.input.args;
    let sought_idx = state.open_markers.len();
    let sought_str = &args.markers[sought_idx];
    let sought_kind = MarkerKind::ALL[sought_idx];

    let source_path = state.input.source.to_string();
    let content = &state.input.content;

    let new_snippet = || Snippet::source(content.as_str()).path(source_path.clone());

    let (ekind, report) = match found_marker {
        Some(marker) => {
            let kind = marker.kind;
            let str = marker.as_str().to_string();
            (
                io::ErrorKind::InvalidData,
                Level::ERROR
                    .primary_title(format!(
                        "unexpected {kind} {str}, expected {sought_kind} {sought_str}"
                    ))
                    .element(
                        new_snippet().annotation(
                            AnnotationKind::Primary
                                .span(marker.span.into())
                                .label(format!("unexpected {kind}")),
                        ),
                    ),
            )
        }
        None => {
            let eof = content.len();
            (
                io::ErrorKind::UnexpectedEof,
                Level::ERROR
                    .primary_title(format!(
                        "unexpected end of file looking for {sought_kind} {sought_str}"
                    ))
                    .element(
                        new_snippet()
                            .annotation(AnnotationKind::Primary.span(eof..eof).label("EOF")),
                    ),
            )
        }
    };

    let prev_markers = prev_marker_annotations(state, &new_snippet);
    let report = report.elements(prev_markers);
    let msg = Renderer::plain().render(&[report]);

    io::Error::new(ekind, msg)
}

/// Construct an [`std::io::Error`] for a validly-sequenced marker which invalidly appears on the
/// same line as its predecessor. See [`]
pub fn err_same_line<'a, 'i>(state: &ParseFileState<'a, 'i>, marker: MarkerInst<'i>) -> io::Error {
    // let args = state.input.args;
    // let sought_idx = state.open_markers.len();
    // let sought_str = &args.markers[sought_idx];
    // let sought_kind = MarkerKind::ALL[sought_idx];

    // let source_path = state.input.source.to_string();
    // let content = &state.input.content;

    // let snippet = || Snippet::source(content.as_str()).path(source_path.clone());

    // let (ekind, report) = match marker {
    //     Some(marker) => {
    //         let kind = marker.kind;
    //         let str = marker.as_str().to_string();
    //         (
    //             io::ErrorKind::InvalidData,
    //             Level::ERROR
    //                 .primary_title(format!(
    //                     "unexpected {kind} {str}, expected {sought_kind} {sought_str}"
    //                 ))
    //                 .element(
    //                     snippet().annotation(
    //                         AnnotationKind::Primary
    //                             .span(marker.span.into())
    //                             .label(format!("unexpected {kind}")),
    //                     ),
    //                 ),
    //         )
    //     }
    //     None => {
    //         let eof = content.len();
    //         (
    //             io::ErrorKind::UnexpectedEof,
    //             Level::ERROR
    //                 .primary_title(format!(
    //                     "unexpected end of file, expected {sought_kind} {sought_str}"
    //                 ))
    //                 .element(
    //                     snippet().annotation(AnnotationKind::Primary.span(eof..eof).label("EOF")),
    //                 ),
    //         )
    //     }
    // };

    // let prev_markers = state
    //     .open_markers
    //     .iter()
    //     .enumerate()
    //     .map(|(i, prev_marker)| {
    //         snippet().annotation(
    //             AnnotationKind::Context
    //                 .span(prev_marker.span.into())
    //                 .label(MarkerKind::ALL[i].description()),
    //         )
    //     });
    // let report = report.elements(prev_markers);
    // let msg = Renderer::plain().render(&[report]);

    // io::Error::new(ekind, msg)
    todo!()
}

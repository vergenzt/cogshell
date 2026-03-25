use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};

use num_traits::FromPrimitive;

use crate::config::{Config, MarkerKind};
use crate::source::SourceMarker;

#[derive(Debug)]
pub struct ParseError<'a> {
    pub config: &'a Config<'a>,
    pub filename: &'a str,
    pub content: &'a str,
    pub marker_found: Option<SourceMarker<'a>>,
    pub parse_state: Vec<SourceMarker<'a>>,
}

impl ParseError<'_> {
    fn print(&self) {
        let marker_sought: &str = &self.config.markers[self.parse_state.len()];
        let marker_sought_desc: &str = MarkerKind::from_usize(self.parse_state.len())
            .unwrap()
            .into();

        let source = Snippet::source(self.content).path(self.filename);
        let state_elements =
            self.parse_state
                .iter()
                .enumerate()
                .map(|(prev_marker_idx, prev_marker)| {
                    source.annotation(
                        AnnotationKind::Context
                            .span(prev_marker.into())
                            .label(prev_marker.kind.into()),
                    )
                });
        let level = Level::ERROR;

        let error = if let Some(marker_found) = &self.marker_found {
            let marker_found_kind = marker_found.kind;
            level.primary_title(format!(
                "unexpected {marker_found_kind} marker {marker_found}, expected {marker_sought_desc} marker {marker_sought}"
            ))
            .element(
                source.clone().annotation(
                    AnnotationKind::Primary
                        .span(marker_found.into())
                        .label(format!("unexpected {marker_found_kind} marker")),
                )
            )
        } else {
            level.primary_title(format!(
                "unexpected end of file, expected {marker_sought_desc} marker {marker_sought}"
            ))
        };

        let report = Renderer::styled().render(&[error]);
        anstream::eprintln!("{}", report);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 1: prog_start and prog_end on the same line — trim whitespace and return
    #[test]
    fn test_rule1_same_line() {
        const CONTENT: &str = "[[[cogsh   echo foo  ]]]\n[[[end]]]";
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(ParsedBlock::extract_program(&block), Ok(vec!["echo foo"]));
    }

    // Rule 2: all lines share a common non-whitespace prefix — strip it, then strip common indent
    #[test]
    fn test_rule2_shared_prefix() {
        const CONTENT: &str = concat!(
            "prologue\n",
            "--[[[cogsh\n",
            "--   echo foo bar baz\n",
            "--   jq -n '1 + 2 + 3'\n",
            "--]]]\n",
            "xy0\n",
            "xy1\n",
            "xy2\n",
            "--[[[end]]]\n",
        );
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(
            ParsedBlock::from(&block),
            Ok(vec!["echo foo bar baz", "jq -n '1 + 2 + 3'"])
        );
    }

    // Rule 3: non-whitespace after prog_start on same line becomes first line (trimmed),
    // remaining lines get common indent stripped (rule 4)
    #[test]
    fn test_rule3_inline_first_line() {
        const CONTENT: &str = concat!(
            "[[[cogsh   first line\n",
            "    second line\n",
            "    third line\n",
            "]]]\n",
            "[[[end]]]",
        );
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(
            ParsedBlock::extract_program(&block),
            Ok(vec!["first line", "second line", "third line"])
        );
    }

    // Rule 4: multiline, no shared non-ws prefix — strip common leading whitespace
    #[test]
    fn test_rule4_common_indent() {
        const CONTENT: &str = concat!(
            "[[[cogsh\n",
            "    echo foo\n",
            "    echo bar\n",
            "]]]\n",
            "[[[end]]]",
        );
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert_eq!(
            ParsedBlock::extract_program(&block),
            Ok(vec!["echo foo", "echo bar"])
        );
    }

    // Error: prog_end and output_end on the same line
    #[test]
    fn test_error_prog_end_output_end_same_line() {
        const CONTENT: &str = "[[[cogsh echo foo]]] [[[end]]]";
        let block = UnparsedBlock {
            content: CONTENT,
            prog_start: 0..0, // TODO
            prog_end: 0..0,   // TODO
            output_end: 0..0, // TODO
        };
        assert!(ParsedBlock::extract_program(&block).is_err());
    }
}

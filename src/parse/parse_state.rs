use std::io;
use std::iter;
use std::ops::Deref as _;

use regex::{CaptureLocations, Regex};

use super::{block::*, errors::*, loc_line::*, marker_inst::*, marker_kind::*, parse_input::*};
use crate::deref_field;

pub(super) struct ParseFileState<'a, 'i> {
    pub input: &'i ParseInput<'a>,
    pub markers_re: Regex,
    pub marker_caps: CaptureLocations,
    pub blocks: Vec<Block<'i>>,
    pub open_markers: Vec<MarkerInst<'i>>,
}

deref_field! {
    impl<'a> *ParseFileState<'a, '_> = .input: ParseInput<'a>
}

impl<'a, 'i> ParseFileState<'a, 'i> {
    pub(super) fn init(input: &'i ParseInput<'a>) -> Self {
        let markers_re = input.markers.get_regex();
        let marker_caps = markers_re.capture_locations();
        Self {
            input,
            markers_re,
            marker_caps,
            blocks: vec![],
            open_markers: vec![],
        }
    }

    pub(super) fn find_markers(&'i mut self, line: &LocatedLine<'i>) -> Vec<MarkerInst<'i>> {
        let Self {
            markers_re: re,
            marker_caps: caps,
            ..
        } = self;
        let mut markers: Vec<MarkerInst<'i>> = vec![];
        loop {
            let last_marker_end = markers.last().map(|m| m.span.end.col).unwrap_or(0);
            if let Some(marker_match) = re.captures_read_at(caps, &*line, last_marker_end) {
                let kind: MarkerKind = caps.into();
                let marker = MarkerInst::new(kind, marker_match, line);
                markers.push(marker);
            } else {
                break markers;
            }
        }
    }

    pub(super) fn add_marker(
        &mut self,
        kind: MarkerKind,
        marker: MarkerInst<'i>,
    ) -> io::Result<()> {
        if kind == self.expected_marker_kind() {
            self.open_markers.push(marker);

            // check for completed block
            if self.open_markers.len() == MarkerKind::ALL.len() {
                let markers = BlockMarkers::new(self.open_markers.as_array().unwrap());
                let lines = &self.lines;
                let block = Block::new(lines, markers);
                self.blocks.push(block);
            }

            Ok(())
        } else {
            return Err(err_unexpected_marker(&self, Some((kind, marker))));
        }
    }

    pub(super) fn expected_marker_kind(&self) -> MarkerKind {
        MarkerKind::ALL[self.open_markers.len()]
    }

    pub(crate) fn finalize_parsed_blocks(self) -> Result<Vec<Block<'i>>, std::io::Error> {
        // unmatched markers left over
        if !self.open_markers.is_empty() {
            return Err(err_unexpected_marker(&self, None));
        }

        Ok(self.blocks)
    }
}

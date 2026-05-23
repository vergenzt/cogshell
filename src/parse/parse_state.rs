use std::io;

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

    /// Find every marker in `line`.
    pub(super) fn find_markers(&mut self, line: &LocatedLine<'i>) -> Vec<MarkerInst<'i>> {
        let re = &self.markers_re;
        let caps = &mut self.marker_caps;
        let mut markers: Vec<MarkerInst<'i>> = vec![];
        loop {
            let last_marker_end = markers.last().map(|m| m.span.end.col).unwrap_or(0);
            match re.captures_read_at(caps, line.1, last_marker_end) {
                Some(mtch) => {
                    let kind: MarkerKind = (&*caps).into();
                    let marker = MarkerInst::new(kind, mtch, line);
                    markers.push(marker);
                }
                None => break markers,
            }
        }
    }

    pub(super) fn add_marker(&mut self, marker: MarkerInst<'i>) -> io::Result<()> {
        if marker.kind == self.expected_marker_kind() {
            self.open_markers.push(marker);

            // check for completed block
            if self.open_markers.len() == MarkerKind::ALL.len() {
                let mut taken = std::mem::take(&mut self.open_markers);
                let arr: [MarkerInst<'i>; 3] = [taken.remove(0), taken.remove(0), taken.remove(0)];
                let markers = BlockMarkers::new(&arr);
                let block = Block::new(&self.input.content, markers);
                self.blocks.push(block);
            }

            Ok(())
        } else {
            Err(err_unexpected_marker(self, Some(marker)))
        }
    }

    pub(super) fn expected_marker_kind(&self) -> MarkerKind {
        MarkerKind::ALL[self.open_markers.len()]
    }

    pub(crate) fn finalize_parsed_blocks(self) -> Result<Vec<Block<'i>>, std::io::Error> {
        if !self.open_markers.is_empty() {
            return Err(err_unexpected_marker(&self, None));
        }
        Ok(self.blocks)
    }
}

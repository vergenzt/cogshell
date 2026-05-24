use std::{io, mem::take};

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

    fn last_marker(&self) -> Option<&MarkerInst<'i>> {
        self.open_markers
            .last()
            .or_else(|| Some(&self.blocks.last()?.markers.2))
    }

    pub(super) fn find_markers(&mut self, line: &LocatedLine<'i>) -> Vec<MarkerInst<'i>> {
        let re = &self.markers_re;
        let caps = &mut self.marker_caps;
        let mut markers: Vec<MarkerInst<'i>> = vec![];
        loop {
            let last_marker_end = markers.last().map(|m| m.span.end.col).unwrap_or(0);
            match re.captures_read_at(caps, line.1, last_marker_end) {
                Some(mtch) => {
                    let kind: MarkerKind = (&*caps).into();
                    let marker = MarkerInst::new(kind, mtch, &line);
                    markers.push(marker);
                }
                None => break markers,
            }
        }
    }

    pub(super) fn push_marker(&mut self, this: MarkerInst<'i>) -> io::Result<()> {
        if this.kind != self.expected_marker_kind() {
            return Err(err_unexpected_marker(self, Some(this)));
        }

        if self.last_marker().is_some_and(|prev| !this.ok_after(prev)) {
            return Err(err_same_line(self, this));
        }

        self.open_markers.push(this);

        // check for completed block
        if self.open_markers.len() == MarkerKind::ALL.len() {
            let new_block = self.finalize_block();
            self.blocks.push(new_block);
        }

        Ok(())
    }

    fn finalize_block(&mut self) -> Block<'i> {
        let content = &self.input.content;
        let markers = BlockMarkers::from({
            let owned = take(&mut self.open_markers);
            *owned.as_array().unwrap()
        });
        Block::from(content, markers)
    }

    pub(super) fn expected_marker_kind(&self) -> MarkerKind {
        MarkerKind::ALL[self.open_markers.len()]
    }

    pub(crate) fn finalize_file(self) -> Result<Vec<Block<'i>>, std::io::Error> {
        if !self.open_markers.is_empty() {
            return Err(err_unexpected_marker(&self, None));
        }
        Ok(self.blocks)
    }
}

use std::io;

use super::{
    block::*, errors::*, loc_line::*, marker_inst::*, marker_kind::*, parse_input::*,
    parse_state::*,
};
use crate::{
    args::{files::*, *},
    deref_field,
};

pub struct ParsedFile<'a, 'i> {
    pub input: &'i ParseInput<'a>,
    pub blocks: Vec<Block<'i>>,
}

deref_field! {
  impl<'a> *ParsedFile<'a, '_> = .input: ParseInput<'a>
}

impl<'a, 'i> ParsedFile<'a, 'i> {
    pub fn from(args: &'a Args, source: &'a mut File<Read>) -> io::Result<Self> {
        let input = &ParseInput::from(args, source)?;

        let mut state = ParseFileState::init(input);

        for line in input.iter_lines() {
            let open_markers = state.open_markers;
            let line_markers = state.find_markers(&line);

            match (state.open_markers[..], ) {
                // no markers on the line, that's fine
                ([..], []) => continue,
                ([], [marker @ MarkerInst { kind: MarkerKind::ProgramStart, ..}]) =>
                Some(marker_match) => {
                    state.add_marker(kind, marker);
                }
            }
        }

        state.finish()
    }
}

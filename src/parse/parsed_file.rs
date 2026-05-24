use std::io;

use super::{block::*, parse_input::*, parse_state::*};
use crate::deref_field;

pub struct ParsedFile<'a, 'i> {
    pub input: &'i ParseInput<'a>,
    pub blocks: Vec<Block<'i>>,
}

deref_field! {
  impl<'a> *ParsedFile<'a, '_> = .input: ParseInput<'a>
}

impl<'a, 'i> ParsedFile<'a, 'i> {
    pub fn from(input: &'i ParseInput<'a>) -> io::Result<Self> {
        let mut state = ParseFileState::init(input);

        for line in input.iter_lines() {
            let markers = state.find_markers(&line);

            // always use the first marker found on a line
            if let [marker, ..] = markers[..] {
                state.push_marker(marker)?;

                for (i, addl_marker) in markers.iter().enumerate().skip(1) {
                    match (&marker.kind, addl_marker.kind) {
                        (MarkerKind::ProgramStart, MarkerKind::ProgramEnd) => {
                            state.push_marker(addl_marker)?
                        }
                    }
                }
            }
        }

        let blocks = state.finalize_file()?;
        Ok(Self { input, blocks })
    }
}

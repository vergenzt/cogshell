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
            for marker in markers {
                state.add_marker(marker)?;
            }
        }

        let blocks = state.finalize_parsed_blocks()?;
        Ok(Self { input, blocks })
    }
}

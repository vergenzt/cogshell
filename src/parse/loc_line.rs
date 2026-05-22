use crate::deref_field;

use super::loc::*;

pub struct LocatedLine<'i>(pub Loc, pub &'i String);

deref_field! { impl<'i> *LocatedLine<'i> = .1: &'i String }

impl<'i> LocatedLine<'i> {
    pub fn iter(lines: &'i Vec<String>) -> impl Iterator<Item = LocatedLine<'i>> {
        lines.iter().scan(Loc::default(), |line_start, line| {
            line_start.line += 1;
            line_start.offset += line.len();
            Some(LocatedLine(*line_start, line))
        })
    }
}

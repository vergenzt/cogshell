use std::io;

use crate::{
    args::{files::*, *},
    deref_field,
    parse::loc_line::LocatedLine,
};

pub(super) struct ParseInput<'a> {
    /// Config parameters used to parse the source
    pub args: &'a Args,
    /// The filename or input stream containing CogShell block(s)
    pub source: &'a File<Read>,
    /// The original inclusively-line-split content read from the source
    pub lines: Vec<String>,
}

deref_field! {
  impl *ParseInput<'_> = .args: Args
}

impl<'a> ParseInput<'a> {
    pub fn from(args: &'a Args, source: &'a mut File<Read>) -> io::Result<Self> {
        let lines = source.read()?;
        Ok(Self {
            args,
            source,
            lines,
        })
    }

    pub fn iter_lines<'i>(&'i self) -> impl Iterator<Item = LocatedLine<'i>> {
        LocatedLine::iter(&self.lines)
    }
}

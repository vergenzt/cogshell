use std::io;

use crate::{
    args::{files::*, *},
    deref_field,
    parse::loc_line::LocatedLine,
};

pub struct ParseInput<'a> {
    /// Config parameters used to parse the source
    pub args: &'a Args,
    /// The filename or input stream containing CogShell block(s)
    pub source: &'a File<Read>,
    /// The original content read from the source
    pub content: String,
}

deref_field! {
  impl *ParseInput<'_> = .args: Args
}

impl<'a> ParseInput<'a> {
    pub fn from(args: &'a Args, source: &'a mut File<Read>) -> io::Result<Self> {
        let content = source.read()?;
        Ok(Self {
            args,
            source,
            content,
        })
    }

    pub fn iter_lines<'i>(&'i self) -> impl Iterator<Item = LocatedLine<'i>> {
        LocatedLine::iter(&self.content)
    }
}

use crate::parse;

pub trait Error: std::error::Error {}

impl Error for std::io::Error {}
impl Error for parse::ParseError<'_> {}

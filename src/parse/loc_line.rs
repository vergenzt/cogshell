use super::loc::*;

pub struct LocatedLine<'i>(pub Loc, pub &'i str);

impl<'i> std::ops::Deref for LocatedLine<'i> {
    type Target = str;
    fn deref(&self) -> &str {
        self.1
    }
}

impl<'i> LocatedLine<'i> {
    pub fn iter(content: &'i str) -> impl Iterator<Item = LocatedLine<'i>> {
        content
            .split_inclusive('\n')
            .scan(Loc::default(), |line_start, line| {
                let this_start = *line_start;
                line_start.offset += line.len();
                line_start.line += 1;
                Some(LocatedLine(this_start, line))
            })
    }
}

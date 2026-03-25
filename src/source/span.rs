use std::ops::Range;

use crate::source::location::SourceLocation;

#[derive(Copy, Clone, Debug)]
pub struct SourceSpan<'a> {
    pub content: &'a str,
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl<'a> SourceSpan<'a> {
    pub fn from_range(
        content: &'a str,
        found: Range<usize>,
        line_start: &mut SourceLocation,
    ) -> Self {
        assert_eq!(line_start.col, 0);

        let start = SourceLocation::from(found.start, SourceLocation { ..*line_start });

        // increment line_start through any newlines in the match
        let lines = content[found.clone()].split_inclusive('\n').enumerate();
        let (last_line_index, last_line) = lines.last().unwrap();
        line_start.line += last_line_index;
        line_start.offset = found.end - last_line.len();

        let end = SourceLocation::from(found.end, SourceLocation { ..*line_start });

        SourceSpan {
            content,
            start,
            end,
        }
    }
}

impl PartialEq for SourceSpan<'_> {
    fn eq(&self, other: &Self) -> bool {
        let [self_addrs, other_addrs] = [self, other].map(|span| {
            let addr = span.content.as_ptr().addr();
            (addr + self.start.offset, addr + self.end.offset)
        });
        self_addrs == other_addrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_span_init() {
        let content = "abcd\nefgh\nijkl\n";
        let mut line_start = SourceLocation::default();
        let actual_span = SourceSpan::from_range(content, 1..4, &mut line_start);
        assert_eq!(
            actual_span,
            SourceSpan {
                content,
                start: SourceLocation {
                    line: 0,
                    col: 1,
                    offset: 1,
                },
                end: SourceLocation {
                    line: 0,
                    col: 5,
                    offset: 5,
                },
            }
        );
        assert_eq!(
            line_start,
            SourceLocation {
                line: 0,
                col: 0,
                offset: 0
            }
        )
    }
}

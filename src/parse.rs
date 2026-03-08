use std::ops::Range;

use anyhow::{bail, Result};
use miette::{Diagnostic, NamedSource};
use regex::Regex;
use thiserror::Error;

use crate::config::Config;
use crate::utils::common_prefix_len;

/// Errors produced when the marker sequence in a file is malformed.
/// Kept for future use with rich miette diagnostics.
#[derive(Error, Diagnostic, Debug)]
#[allow(dead_code)]
pub enum ParseError {
    #[error("found {mark:?} with no opening {prog_start:?}")]
    ExpectedProgStart {
        #[source_code]
        src: NamedSource<String>,
        #[label("found")]
        mark: Range<usize>,
        prog_start: String,
    },
    #[error("expected {prog_end:?} following {prog_start_marker:?}, but got {mark:?}")]
    ExpectedProgEnd {
        #[source_code]
        src: NamedSource<String>,
        #[label(primary, "expected program close")]
        mark: Range<usize>,
        #[label("after program start here")]
        prog_start_marker: Range<usize>,
        prog_end: String,
    },
    #[error("expected {output_end:?} following program start and end, but got {mark:?}")]
    ExpectedOutputEnd {
        #[source_code]
        src: NamedSource<String>,
        #[label(primary, "expected output close")]
        mark: Range<usize>,
        #[label("after program start here")]
        prog_start_marker: Range<usize>,
        #[label("and program end here")]
        prog_end: Range<usize>,
        output_end: String,
    },
}

pub struct ParsedFile<'a> {
    /// The original content of the file
    pub content: &'a str,
    /// Config used to parse the file
    pub config: &'a Config<'a>,
    /// The CogShell code blocks parsed from the file
    pub blocks: Vec<ParsedBlock>,
}

impl ParsedFile<'_> {
    pub fn from_content<'a>(
        content: &'a str,
        config: &'a Config,
        name: Option<&str>,
    ) -> Result<ParsedFile<'a>> {
        let [prog_start_str, prog_end_str, output_end_str] = config.markers;

        let pattern = config
            .markers
            .iter()
            .map(|m| regex::escape(m))
            .collect::<Vec<_>>()
            .join("|");
        let marker_re = Regex::new(&pattern)?;

        let markers_found: Vec<_> = marker_re.find_iter(content).collect();
        let src_name = name.unwrap_or("<unknown>");

        let mut blocks = Vec::new();
        let mut i = 0;

        while i < markers_found.len() {
            let a = &markers_found[i];
            if a.as_str() != prog_start_str {
                bail!(
                    "{}:{}: found {:?} where {:?} was expected",
                    src_name,
                    a.start(),
                    a.as_str(),
                    prog_start_str,
                );
            }

            let b = markers_found.get(i + 1).ok_or_else(|| {
                anyhow::anyhow!(
                    "{}:{}: {:?} opened but never closed with {:?}",
                    src_name,
                    a.start(),
                    prog_start_str,
                    prog_end_str,
                )
            })?;
            if b.as_str() != prog_end_str {
                bail!(
                    "{}:{}: expected {:?} but got {:?}",
                    src_name,
                    b.start(),
                    prog_end_str,
                    b.as_str(),
                );
            }

            let c = markers_found.get(i + 2).ok_or_else(|| {
                anyhow::anyhow!(
                    "{}:{}: {:?} opened but never closed with {:?}",
                    src_name,
                    b.start(),
                    prog_end_str,
                    output_end_str,
                )
            })?;
            if c.as_str() != output_end_str {
                bail!(
                    "{}:{}: expected {:?} but got {:?}",
                    src_name,
                    c.start(),
                    output_end_str,
                    c.as_str(),
                );
            }

            blocks.push(ParsedBlock::from(content, &[a.range(), b.range(), c.range()])?);
            i += 3;
        }

        Ok(ParsedFile { content, config, blocks })
    }
}

/// Everything needed to execute an embedded code block.
pub struct ParsedBlock {
    /// The pre-trimmed lines of the program to run.
    pub program: Vec<String>,
    /// The whitespace prefix to prepend to each output line (taken from the indentation
    /// before the opening marker on its source line).
    pub program_common_whitespace: String,
}

impl ParsedBlock {
    /// Extract a [`ParsedBlock`] from the three marker ranges within `content`.
    ///
    /// `markers` is `[prog_start, prog_end, output_end]` — the byte ranges of the three
    /// marker strings as they appear in `content`.
    ///
    /// The program text is extracted according to four rules:
    ///
    /// - **Rule 1**: `prog_start` and `prog_end` on the same line → trim whitespace.
    /// - **Rule 2**: all lines from the `prog_start` line through the `prog_end` line share a
    ///   common *non-whitespace* prefix → strip that prefix, then strip common indentation from
    ///   the interior program lines.
    /// - **Rule 3**: there is non-whitespace content after `prog_start` on its line (but the
    ///   block is multiline) → that content becomes the first line (trimmed), and the remaining
    ///   lines have their common indentation stripped.
    /// - **Rule 4**: multiline, none of the above → strip common leading whitespace.
    ///
    /// It is always an error for `prog_end` and `output_end` to be on the same line.
    pub fn from(content: &str, markers: &[Range<usize>; 3]) -> Result<ParsedBlock> {
        let [prog_start, prog_end, output_end] = markers;

        // It is always an error for ]]] and [[[end]]] to share a line — there would be no room
        // for the output section.
        let between_end_markers = &content[prog_end.end()..output_end.start()];
        if !between_end_markers.contains('\n') {
            bail!(
                "prog_end ({:?}) and output_end ({:?}) must be on different lines",
                &content[prog_end.clone()],
                &content[output_end.clone()],
            );
        }

        // The whitespace prefix before the opening marker on its line becomes the prefix that
        // CogShell prepends to each generated output line.
        let line_start = content[..prog_start.start()]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let pre_marker = &content[line_start..prog_start.start()];
        let program_common_whitespace = pre_marker
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>();

        let program_raw = &content[prog_start.end()..prog_end.start()];

        if !program_raw.contains('\n') {
            // Rule 1: prog_start and prog_end are on the same line.
            return Ok(ParsedBlock {
                program: vec![program_raw.trim().to_string()],
                program_common_whitespace,
            });
        }

        // Multiline: check whether there is non-whitespace content on the same line as
        // prog_start (Rule 3).
        let first_newline = program_raw.find('\n').unwrap();
        let first_line_content = &program_raw[..first_newline];

        if !first_line_content.trim().is_empty() {
            // Rule 3: inline first line.
            let remaining = &program_raw[first_newline + 1..];
            let remaining_lines: Vec<&str> = remaining.lines().collect();
            let common_indent = if remaining_lines.is_empty() {
                0
            } else {
                common_prefix_len(&remaining_lines, true)
            };
            let mut program = vec![first_line_content.trim().to_string()];
            for line in &remaining_lines {
                program.push(
                    line.get(common_indent..)
                        .unwrap_or_else(|| line.trim_end())
                        .to_string(),
                );
            }
            return Ok(ParsedBlock { program, program_common_whitespace });
        }

        // Check for a shared non-whitespace prefix across all lines from the prog_start line
        // through the prog_end line (Rule 2).
        let prog_end_line_end = content[prog_end.end()..]
            .find('\n')
            .map(|i| prog_end.end() + i)
            .unwrap_or(content.len());
        let section = &content[line_start..prog_end_line_end];
        let section_lines: Vec<&str> = section.lines().collect();

        let common_prefix = common_prefix_len(&section_lines, false);
        let has_nonws_prefix = common_prefix > 0
            && !section_lines[0][..common_prefix].trim().is_empty();

        if has_nonws_prefix {
            // Rule 2: strip the common non-whitespace prefix, then strip common indentation
            // from the interior program lines (everything except the first and last section
            // lines, which contain the markers).
            let stripped: Vec<&str> = section_lines
                .iter()
                .map(|l| l.get(common_prefix..).unwrap_or(""))
                .collect();
            let program_lines = &stripped[1..stripped.len().saturating_sub(1)];
            let common_indent = if program_lines.is_empty() {
                0
            } else {
                common_prefix_len(program_lines, true)
            };
            let program = program_lines
                .iter()
                .map(|l| {
                    l.get(common_indent..)
                        .unwrap_or_else(|| l.trim_end())
                        .to_string()
                })
                .collect();
            return Ok(ParsedBlock { program, program_common_whitespace });
        }

        // Rule 4: strip common leading whitespace from all non-empty program lines.
        let program_lines: Vec<&str> = program_raw
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        let common_indent = if program_lines.is_empty() {
            0
        } else {
            common_prefix_len(&program_lines, true)
        };
        let program = program_lines
            .iter()
            .map(|l| {
                l.get(common_indent..)
                    .unwrap_or_else(|| l.trim_end())
                    .to_string()
            })
            .collect();
        Ok(ParsedBlock { program, program_common_whitespace })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rule 1: prog_start and prog_end on the same line — trim whitespace and return
    #[test]
    fn test_rule1_same_line() {
        let content = "[[[cogsh   echo foo  ]]]\n[[[end]]]";
        let config = Config::default();
        let file = ParsedFile::from_content(content, &config, None).unwrap();
        assert_eq!(file.blocks.len(), 1);
        assert_eq!(file.blocks[0].program, vec!["echo foo"]);
    }

    // Rule 2: all lines share a common non-whitespace prefix — strip it, then strip common indent
    #[test]
    fn test_rule2_shared_prefix() {
        let content = concat!(
            "prologue\n",
            "--[[[cogsh \n",
            "--   echo foo bar baz\n",
            "--   jq -n '1 + 2 + 3'\n",
            "--]]]\n",
            "xy0\n",
            "xy1\n",
            "xy2\n",
            "--[[[end]]]\n",
        );
        let config = Config::default();
        let file = ParsedFile::from_content(content, &config, None).unwrap();
        assert_eq!(file.blocks.len(), 1);
        assert_eq!(
            file.blocks[0].program,
            vec!["echo foo bar baz", "jq -n '1 + 2 + 3'"],
        );
    }

    // Rule 3: non-whitespace after prog_start on same line becomes first line (trimmed),
    // remaining lines get common indent stripped (rule 4)
    #[test]
    fn test_rule3_inline_first_line() {
        let content = concat!(
            "[[[cogsh   first line\n",
            "    second line\n",
            "    third line\n",
            "]]]\n",
            "[[[end]]]",
        );
        let config = Config::default();
        let file = ParsedFile::from_content(content, &config, None).unwrap();
        assert_eq!(file.blocks.len(), 1);
        assert_eq!(
            file.blocks[0].program,
            vec!["first line", "second line", "third line"],
        );
    }

    // Rule 4: multiline, no shared non-ws prefix — strip common leading whitespace
    #[test]
    fn test_rule4_common_indent() {
        let content = concat!(
            "[[[cogsh \n",
            "    echo foo\n",
            "    echo bar\n",
            "]]]\n",
            "[[[end]]]",
        );
        let config = Config::default();
        let file = ParsedFile::from_content(content, &config, None).unwrap();
        assert_eq!(file.blocks.len(), 1);
        assert_eq!(file.blocks[0].program, vec!["echo foo", "echo bar"]);
    }

    // Error: prog_end and output_end on the same line
    #[test]
    fn test_error_prog_end_output_end_same_line() {
        let content = "[[[cogsh echo foo]]] [[[end]]]";
        let config = Config::default();
        assert!(ParsedFile::from_content(content, &config, None).is_err());
    }
}

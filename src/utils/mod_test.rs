use super::*;
use yare::parameterized;

#[parameterized(

  // general prefixes

  no_lines = {
    (vec![], |_| true),
    "",
  },

  single_line_any_char = {
    (vec!["hello world"], |_| true),
    "hello world",
  },

  no_common = {
    (vec!["abc", "xyz"], |_| true),
    "",
  },

  one_is_prefix_of_other = {
    (vec!["foo", "foobar"], |_| true),
    "foo",
  },

  other_is_prefix_of_one = {
    (vec!["foobar", "foo"], |_| true),
    "foo",
  },

  all_identical = {
    (vec!["hello", "hello", "hello"], |_| true),
    "hello",
  },

  with_empty_string = {
    (vec!["foo", "", "bar"], |_| true),
    "",
  },

  utf8_multibyte = {
    (vec!["café latte", "cafå mocha"], |_| true),
    "caf",
  },

  // with character filters

  whitespace_single_line = {
    (vec!["    hello world"], |c: char| c.is_ascii_whitespace()),
    "    ",
  },

  whitespace_all_indented = {
    (vec!["    asdf", "    jkl", "    foo"], |c: char| c.is_ascii_whitespace()),
    "    ",
  },

  whitespace_increasing = {
    (vec!["  asdf", "    jkl", "      foo"], |c: char| c.is_ascii_whitespace()),
    "  ",
  },

  whitespace_with_empty_line = {
    (vec!["  asdf", "    jkl", "", "      foo"], |c: char| c.is_ascii_whitespace()),
    "",
  },

  whitespace_mixed = {
    (vec!["   \tasdf", "\t jkl", "", "      foo"], |c: char| c.is_ascii_whitespace()),
    "",
  },

)]
fn test_common_prefix_general(inputs: _, expected: &str) {
    assert_eq!(common_prefix_of_chars(inputs.0, inputs.1), expected);
}

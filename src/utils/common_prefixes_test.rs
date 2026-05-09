use super::common_prefixes::*;
use yare::parameterized;

#[parameterized(

  // general prefixes

  no_lines = {
    vec![],
    None,
  },

  single_line_any_char = {
    vec!["hello world"],
    None,
  },

  no_common = {
    vec!["abc", "xyz"],
    Some(""),
  },

  one_is_prefix_of_other = {
    vec!["foo", "foobar"],
    Some("foo"),
  },

  other_is_prefix_of_one = {
    vec!["foobar", "foo"],
    Some("foo"),
  },

  all_identical = {
    vec!["hello", "hello", "hello"],
    Some("hello"),
  },

  with_empty_string = {
    vec!["foo", "", "bar"],
    Some(""),
  },

  utf8_multibyte = {
    vec!["café latte", "cafå mocha"],
    Some("caf"),
  },

  // with character filters

  whitespace_single_line = {
    vec!["    hello world"],
    Some("    "),
  },

  whitespace_all_indented = {
    vec!["    asdf", "    jkl&b", "    foo"],
    Some("    "),
  },

  whitespace_increasing = {
    vec!["  asdf", "    jkl", "      foo"],
    Some("  "),
  },

  whitespace_with_empty_line = {
    vec!["  asdf", "    jkl", "", "      foo"],
    Some(""),
  },

  whitespace_mixed = {
    vec!["   \tasdf", "\t jkl", "", "      foo"],
    Some(""),
  },

)]
fn test_common_prefix_general(inputs: Vec<&str>, expected: Option<&str>) {
    assert_eq!(common_prefix_of_chars(&inputs), expected);
}

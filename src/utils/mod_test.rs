use super::*;
use yare::parameterized;

#[parameterized(

  // general prefixes

  no_lines = {
    vec![],
    None,
  },

  single_line_any_char = {
    vec![b"hello world"[..]],
    None,
  },

  no_common = {
    vec![b"abc"[..], b"xyz"[..]],
    Some(b""[..]),
  },

  one_is_prefix_of_other = {
    vec![b"foo"[..], b"foobar"[..]],
    Some(b"foo"[..]),
  },

  other_is_prefix_of_one = {
    vec![b"foobar"[..], b"foo"[..]],
    Some(b"foo"[..]),
  },

  all_identical = {
    vec![b"hello"[..], b"hello"[..], b"hello"[..]],
    Some(b"hello"[..]),
  },

  with_empty_string = {
    vec![b"foo"[..], b""[..], b"bar"[..]],
    Some(b""[..]),
  },

  utf8_multibyte = {
    vec!["café latte".as_bytes(), "cafå mocha".as_bytes()],
    Some(b"caf"[..]),
  },

  // with character filters

  whitespace_single_line = {
    vec![b"    hello world"[..]],
    Some(b"    "[..]),
  },

  whitespace_all_indented = {
    vec![b"    asdf"[..], b"    jklb"[..], b"    foo"[..]],
    Some(b"    "[..]),
  },

  whitespace_increasing = {
    vec![b"  asdf"[..], b"    jkl"[..], b"      foo"[..]],
    Some(b"  "[..]),
  },

  whitespace_with_empty_line = {
    vec![b"  asdf"[..], b"    jkl"[..], b""[..], b"      foo"[..]],
    Some(b""[..]),
  },

  whitespace_mixed = {
    vec![b"   \tasdf"[..], b"\t jkl"[..], b""[..], b"      foo"[..]],
    Some(b""[..]),
  },

)]
fn test_common_prefix_general(inputs: Vec<ByteStr>, expected: Option<ByteStr>) {
    assert_eq!(common_prefix_of_chars(inputs), expected);
}

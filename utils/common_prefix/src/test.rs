use super::common_prefixes::*;

macro_rules! test_common_prefixes {
    ($label:ident : $inputs:expr => $result:expr) => {
      #[test]
      fn ${concat(test_common_prefix_, $label)}() {
          assert_eq!(common_prefix_of_chars(&$inputs), $result);
      }
    };
}

test_common_prefixes! {
  no_lines:
  vec![] => None
}

test_common_prefixes! {
  single_line_any_char:
  vec!["hello world"] => None
}

test_common_prefixes! {
  no_common:
  vec!["abc", "xyz"] => Some("")

}

test_common_prefixes! {
  one_is_prefix_of_other:
  vec!["foo", "foobar"] => Some("foo")
}

test_common_prefixes! {
  other_is_prefix_of_one:
  vec!["foobar", "foo"] => Some("foo")
}

test_common_prefixes! {
  all_identical:
  vec!["hello", "hello", "hello"] => Some("hello")
}

test_common_prefixes! {
  with_empty_string:
  vec!["foo", "", "bar"] => Some("")
}

test_common_prefixes! {
  utf8_multibyte:
  vec!["café latte", "cafå mocha"] => Some("caf")
}

test_common_prefixes! {
  whitespace_single_line:
  vec!["    hello world"] => Some("    ")
}

test_common_prefixes! {
  whitespace_all_indented:
  vec!["    asdf", "    jkl&b", "    foo"] => Some("    ")
}

test_common_prefixes! {
  whitespace_increasing:
  vec!["  asdf", "    jkl", "      foo"] => Some("  ")
}

test_common_prefixes! {
  whitespace_with_empty_line:
  vec!["  asdf", "    jkl", "", "      foo"] => Some("")
}

test_common_prefixes! {
  whitespace_mixed:
  vec!["   \tasdf", "\t jkl", "", "      foo"] => Some("")
}

use std::{fmt::Display, str::FromStr};

use regex::Regex;
use regex_syntax::is_word_character;

use crate::deref_field;

#[derive(Clone, Debug)]
pub struct MarkerDefs([String; 3]);

impl MarkerDefs {
    pub fn get_regex(&self) -> regex::Regex {
        let groups = self.each_ref().map(|m| {
            // if first/last char is a word char, expect it to follow/precede a word boundary
            let [head_bound, tail_bound] = [m.chars().next(), m.chars().last()]
                .map(Option::unwrap)
                .map(|c| if is_word_character(c) { r"\b" } else { "" });
            let marker_escaped = regex::escape(m);
            format!(r"({head_bound}{marker_escaped}{tail_bound})")
        });
        Regex::new(&groups.join("|")).unwrap()
    }
}

impl FromStr for MarkerDefs {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split_whitespace().collect();
        match parts.as_array::<3>() {
            Some(arr) => Ok(Self(arr.map(String::from))),
            None => Err(format!(
                "Marker config `{s}` does not consist of three whitespace-separated words!"
            )),
        }
    }
}

deref_field! { impl *MarkerDefs = .0: [String; 3] }

impl Display for MarkerDefs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c] = &self.0;
        write!(f, "{a} {b} {c}")
    }
}

use std::sync::LazyLock;

use regex::{Match, Regex};

#[derive(Copy, Clone, Debug)]
pub enum OutputHashKind {
    Md5Hex,
    Md5Base64,
}

impl OutputHashKind {
    fn label(&self) -> &'static str {
        match self {
            // both from cog
            Self::Md5Hex => &"checksum",
            Self::Md5Base64 => &"sum",
        }
    }

    fn from_label(label: &str) -> Option<OutputHashKind> {
        match label {
            l if l == Self::Md5Hex.label() => Some(Self::Md5Hex),
            l if l == Self::Md5Base64.label() => Some(Self::Md5Base64),
            _ => None,
        }
    }
}

static OUTPUT_HASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
          \A # must start immediately
          [ ]*
          \(
            (?<kind> checksum | sum )
            :
            (?<hash> [a-zA-Z0-9+/-_]+ )
          \)
        ",
    )
    .unwrap()
});

pub struct OutputHash<'a> {
    kind: OutputHashKind,
    hash: Match<'a>,
}

impl<'a> OutputHash<'a> {
    pub fn from_block_suffix(block_sfx: &'a str) -> Option<OutputHash<'a>> {
        let caps = OUTPUT_HASH_RE.captures(block_sfx)?;
        let kind = OutputHashKind::from_label(caps.name("kind")?.as_str())?;
        let hash = caps.name("hash")?;
        Some(Self { kind, hash })
    }

    pub fn matches(&self, output: &str) -> bool {
        let
        let Self { kind, hash } = self;
    }
}

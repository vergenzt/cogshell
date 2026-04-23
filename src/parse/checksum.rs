use std::sync::LazyLock;

use base64::{Engine as _, prelude::BASE64_STANDARD};
use hex::ToHex;
use regex::bytes::Regex;

#[derive(Copy, Clone, Debug)]
pub enum ChecksumKind {
    Md5Hex,
    Md5Base64Prefix10Chars,
}

impl ChecksumKind {
    fn label(&self) -> &'static [u8] {
        match self {
            Self::Md5Hex => "checksum".as_bytes(),
            Self::Md5Base64Prefix10Chars => "sum".as_bytes(),
        }
    }

    fn from_label(label: &[u8]) -> Option<ChecksumKind> {
        match label {
            l if l == Self::Md5Hex.label() => Some(Self::Md5Hex),
            l if l == Self::Md5Base64Prefix10Chars.label() => Some(Self::Md5Base64Prefix10Chars),
            _ => None,
        }
    }
}

static CHECKSUM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x-u)
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

pub struct Checksum<'a> {
    kind: ChecksumKind,
    hash: &'a [u8],
}

impl<'a> Checksum<'a> {
    /// Search for an output hash suffix following a CogShell block, given the str starting immediately after output end mark
    pub fn from_block_suffix(block_sfx: &'a [u8]) -> Option<Checksum<'a>> {
        let caps = CHECKSUM_RE.captures(block_sfx)?;
        let kind = ChecksumKind::from_label(caps.name("kind")?.as_bytes())?;
        let hash = caps.name("hash")?.as_bytes();
        Some(Self { kind, hash })
    }

    /// Validate this saved output hash against the output
    pub fn matches(&self, output: &[u8]) -> bool {
        let hash_computed = md5::compute(output);
        let hash_comp_str: &[u8] = match self.kind {
            ChecksumKind::Md5Hex => hash_computed,
            ChecksumKind::Md5Base64Prefix10Chars => {
                let mut hash_comp_b64 = BASE64_STANDARD.encode(&hash_computed.0);
                &hash_comp_b64.as_bytes()[..10]
            }
        };
        self.hash == hash_comp_str
    }
}

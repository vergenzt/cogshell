use std::fmt::{self, Display};
use std::io::{Read, read_to_string};
use std::str::FromStr;

use anyhow::{Result, ensure};
use auditable_serde::VersionInfo;
use flate2::Compression;
use flate2::read::{ZlibDecoder, ZlibEncoder};
use serde::Serialize;
use wasm_encoder::{ComponentSection, CustomSection, Encode, Section};
use wasmparser::CustomSectionReader;

/// Human-readable description of the binary
#[derive(Debug, Clone, PartialEq)]
pub struct Dependencies {
    version_info: VersionInfo,
    custom_section: CustomSection<'static>,
}

impl Dependencies {
    /// Parse an `description` custom section from a wasm binary.
    pub(crate) fn parse_custom_section(reader: &CustomSectionReader<'_>) -> Result<Self> {
        ensure!(
            reader.name() == ".dep-v0",
            "The `dependencies` custom section should have a name of '.dep-v0'"
        );
        let decompressed_data = read_to_string(ZlibDecoder::new(reader.data()))?;
        let dependency_tree = auditable_serde::VersionInfo::from_str(&decompressed_data)?;

        Ok(Self {
            version_info: dependency_tree,
            custom_section: CustomSection {
                name: ".dep-v0".into(),
                data: reader.data().to_owned().into(),
            },
        })
    }

    /// Create a new instance of `Dependencies`.
    pub fn new(dependency_tree: auditable_serde::VersionInfo) -> Self {
        let data = serde_json::to_string(&dependency_tree).unwrap();

        let mut ret_vec = Vec::new();
        let mut encoder = ZlibEncoder::new(data.as_bytes(), Compression::fast());
        encoder.read_to_end(&mut ret_vec).unwrap();

        Self {
            version_info: dependency_tree,
            custom_section: CustomSection {
                name: ".dep-v0".into(),
                data: ret_vec.into(),
            },
        }
    }

    /// Provides access to the version information stored in the object
    pub fn version_info(&self) -> &VersionInfo {
        &self.version_info
    }
}

impl Serialize for Dependencies {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Display for Dependencies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: this will never panic since we always guarantee the data is
        // encoded as utf8, even if we internally store it as [u8].
        // let data = String::from_utf8(self.0.data.to_vec()).unwrap();
        let data = serde_json::to_string(&self.version_info).unwrap();
        write!(f, "{data}")
    }
}

impl ComponentSection for Dependencies {
    fn id(&self) -> u8 {
        ComponentSection::id(&self.custom_section)
    }
}

impl Section for Dependencies {
    fn id(&self) -> u8 {
        Section::id(&self.custom_section)
    }
}

impl Encode for Dependencies {
    fn encode(&self, sink: &mut Vec<u8>) {
        self.custom_section.encode(sink);
    }
}



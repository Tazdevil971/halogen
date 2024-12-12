use serde::{Deserialize, Serialize};

fn u64_is_zero(value: &u64) -> bool {
    *value == 0
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Access {
    #[serde(rename = "rw")]
    ReadWrite,
    #[serde(rename = "ro")]
    Read,
    #[serde(rename = "wo")]
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Array {
    pub len: u64,
    pub stride: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MultiChip {
    pub chips: Vec<Chip>,
    pub modules: Vec<Module>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Chip {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub peripherals: Vec<chip::Peripheral>,
    pub imports: Vec<chip::Import>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arm_ext: Option<chip::ArmExt>,
}

pub mod chip {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub struct ArmExt {
        pub core_name: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub struct Peripheral {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub module: String,
        pub block: String,
        pub address: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub struct Import {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub version: Option<String>,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Module {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub blocks: Vec<Block>,
    pub bitfields: Vec<Bitfield>,
    pub enums: Vec<Enum>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Block {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fields: Vec<block::Field>,
}

pub mod block {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub struct Field {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub array: Option<Array>,
        pub byte_offset: u64,
        #[serde(flatten)]
        pub inner: FieldInner,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum FieldInner {
        Block(field::Block),
        Bitfield(field::Bitfield),
        Simple(field::Simple),
    }

    pub mod field {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
        pub struct Block {
            pub block_name: String,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
        pub struct Bitfield {
            pub access: Access,
            pub bitfield_name: String,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
        pub struct Simple {
            pub access: Access,
            pub bit_size: u32,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub enum_name: Option<String>,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Bitfield {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub bit_size: u32,
    #[serde(default, skip_serializing_if = "u64_is_zero")]
    pub default: u64,
    pub fields: Vec<bitfield::Field>,
}

pub mod bitfield {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub struct Field {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub bit_offset: u32,
        pub bit_size: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub array: Option<Array>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub enum_name: Option<String>,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Enum {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub bit_size: u32,
    pub variants: Vec<enum_name::Variant>,
}

pub mod enum_name {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
    pub struct Variant {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub value: u64,
    }
}

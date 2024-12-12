use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Access {
    ReadWrite,
    Read,
    Write,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Array {
    pub len: u64,
    pub stride: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MultiChip {
    pub chips: Vec<Chip>,
    pub modules: Vec<Module>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Chip {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub modules: Vec<ModuleOrImport>,
    pub peripherals: Vec<chip::Peripheral>,
}

pub mod chip {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Peripheral {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub module: String,
        pub block: String,
        pub address: u64
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum ModuleOrImport {
    Import(Import),
    #[serde(untagged)]
    Module(Module),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Import {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Block {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fields: Vec<block::Field>,
}

pub mod block {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
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
    
    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum FieldInner {
        Block(field::Block),
        Bitfield(field::Bitfield),
        Simple(field::Simple),
    }
    
    pub mod field {
        use super::*;

        #[derive(Debug, Clone, Deserialize, Serialize)]
        pub struct Block {
            pub block_name: String,
        }
        
        #[derive(Debug, Clone, Deserialize, Serialize)]
        pub struct Bitfield {
            pub access: Access,
            pub bitfield_name: String,
        }
    
        #[derive(Debug, Clone, Deserialize, Serialize)]
        pub struct Simple {
            pub access: Access,
            pub bit_size: u32,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            #[serde(rename = "enum")]
            pub enumm: Option<String>,
        }
    }

}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bitfield {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub bit_size: u32,
    pub fields: Vec<bitfield::Field>,
}

pub mod bitfield {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Field {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub bit_offset: u32,
        pub bit_size: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub array: Option<Array>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[serde(rename = "enum")]
        pub enumm: Option<String>,
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Enum {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub bit_size: u32,
    pub variants: Vec<enumm::Variant>,
}

pub mod enumm {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Variant {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        pub value: u64,
    }
}
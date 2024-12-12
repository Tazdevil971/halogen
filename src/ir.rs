use serde::{Deserialize, Serialize};

use std::collections::HashMap;

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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Root {
    pub device: Vec<Device>,
    pub modules: HashMap<String, Module>
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct BlockRef {
    pub module_name: String,
    pub block_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Device {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub peripherals: Vec<device::Peripheral>
}

pub mod device {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Peripheral {
        pub name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub array: Option<Array>,
        pub address: u64,
        pub block_ref: BlockRef
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Module {
    pub name: String,
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
            pub bitfield_name: String,
        }
    
        #[derive(Debug, Clone, Deserialize, Serialize)]
        pub struct Simple {
            pub access: Access,
            pub bit_size: u32,
            #[serde(default, skip_serializing_if = "Option::is_none")]
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
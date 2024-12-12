use anyhow::{ensure, Context as _, Result};
use chiptool::ir as cir;
use log::warn;

use crate::ir;

pub enum Format {
    Yaml,
    Json,
}

pub fn convert_chiptool_source(
    name: String,
    version: Option<String>,
    source: &str,
    format: Format,
) -> Result<ir::Module> {
    let data: cir::IR = match format {
        Format::Yaml => serde_yml::from_str(source).context("failed to parse chiptool yaml")?,
        Format::Json => serde_json::from_str(source).context("failed to parse chiptool json")?,
    };

    convert_chiptool(name, version, data)
}

pub fn convert_chiptool(
    name: String,
    version: Option<String>,
    mut data: cir::IR,
) -> Result<ir::Module> {
    let res = chiptool::validate::validate(
        &data,
        chiptool::validate::Options {
            allow_register_overlap: true,
            allow_field_overlap: true,
            allow_enum_dup_value: true,
            allow_unused_enums: true,
            allow_unused_fieldsets: true,
        },
    );

    ensure!(
        res.is_empty(),
        "chiptool validation failed:\n{}",
        res.join("\n")
    );

    // Expand all definitions
    chiptool::transform::expand_extends::ExpandExtends {}.run(&mut data)?;

    let mut out = ir::Module {
        name,
        version,
        description: None,
        blocks: Vec::new(),
        bitfields: Vec::new(),
        enums: Vec::new(),
    };

    for (name, block) in data.blocks {
        out.blocks.push(convert_block(name, block));
    }

    for (name, fieldset) in data.fieldsets {
        out.bitfields.push(convert_fieldset(name, fieldset));
    }

    for (name, enumm) in data.enums {
        out.enums.push(convert_enum(name, enumm));
    }

    Ok(out)
}

fn convert_block(name: String, block: cir::Block) -> ir::Block {
    let mut fields = Vec::new();
    for item in block.items {
        let array = match item.array {
            Some(cir::Array::Regular(array)) => Some(ir::Array {
                len: array.len as u64,
                stride: array.stride as u64,
            }),
            Some(cir::Array::Cursed(_)) => {
                warn!("skipped cursed array field {} in block {}", item.name, name);
                continue;
            }
            None => None,
        };

        let inner = match item.inner {
            cir::BlockItemInner::Block(block) => {
                ir::block::FieldInner::Block(ir::block::field::Block {
                    block_name: block.block,
                })
            }
            cir::BlockItemInner::Register(reg) => {
                if let Some(fieldset) = reg.fieldset {
                    ir::block::FieldInner::Bitfield(ir::block::field::Bitfield {
                        access: match reg.access {
                            cir::Access::Read => ir::Access::Read,
                            cir::Access::Write => ir::Access::Write,
                            cir::Access::ReadWrite => ir::Access::ReadWrite,
                        },
                        bitfield_name: fieldset,
                    })
                } else {
                    ir::block::FieldInner::Simple(ir::block::field::Simple {
                        access: match reg.access {
                            cir::Access::Read => ir::Access::Read,
                            cir::Access::Write => ir::Access::Write,
                            cir::Access::ReadWrite => ir::Access::ReadWrite,
                        },
                        bit_size: reg.bit_size,
                        enumm: None,
                    })
                }
            }
        };

        fields.push(ir::block::Field {
            name: item.name,
            description: item.description,
            array,
            byte_offset: item.byte_offset as u64,
            inner,
        });
    }

    ir::Block {
        name,
        description: block.description,
        fields,
    }
}

fn convert_fieldset(name: String, fieldset: cir::FieldSet) -> ir::Bitfield {
    let mut fields = Vec::new();
    for field in fieldset.fields {
        let array = match field.array {
            Some(cir::Array::Regular(array)) => Some(ir::Array {
                len: array.len as u64,
                stride: array.stride as u64,
            }),
            Some(cir::Array::Cursed(_)) => {
                warn!(
                    "skipped cursed array field {} in fieldset {}",
                    field.name, name
                );
                continue;
            }
            None => None,
        };

        let bit_offset = match field.bit_offset {
            cir::BitOffset::Regular(bit_offset) => bit_offset,
            cir::BitOffset::Cursed(_) => {
                warn!(
                    "skipped cursed bit offset field {} in fieldset {}",
                    field.name, name
                );
                continue;
            }
        };

        fields.push(ir::bitfield::Field {
            name: field.name,
            description: field.description,
            array,
            bit_offset,
            bit_size: field.bit_size,
            enumm: field.enumm,
        });
    }

    ir::Bitfield {
        name,
        description: fieldset.description,
        bit_size: fieldset.bit_size,
        fields,
    }
}

fn convert_enum(name: String, enumm: cir::Enum) -> ir::Enum {
    let variants = enumm
        .variants
        .into_iter()
        .map(|variant| ir::enumm::Variant {
            name: variant.name,
            description: variant.description,
            value: variant.value,
        })
        .collect();

    ir::Enum {
        name,
        description: enumm.description,
        bit_size: enumm.bit_size,
        variants,
    }
}

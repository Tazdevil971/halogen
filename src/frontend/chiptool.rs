use anyhow::{bail, Context as _, Result};
use chiptool::ir as cir;
use log::warn;

use crate::ir;

pub enum Format {
    Yaml,
    Json,
}

pub fn convert_chiptool(
    name: String,
    source: &str,
    format: Format,
    mut blocks: Vec<String>,
) -> Result<ir::Module> {
    let mut data: cir::IR = match format {
        Format::Yaml => serde_yml::from_str(source).context("failed to parse yaml")?,
        Format::Json => serde_json::from_str(source).context("failed to parse json")?,
    };

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

    if res.len() > 0 {
        bail!("chiptool validation failed:\n{}", res.join("\n"));
    }

    // Expand all definitions
    chiptool::transform::expand_extends::ExpandExtends {}.run(&mut data)?;

    let mut out = ir::Module {
        name,
        description: None,
        blocks: Vec::new(),
        bitfields: Vec::new(),
        enums: Vec::new(),
    };

    let mut fieldsets = Vec::new();
    let mut enums = Vec::new();

    while let Some(name) = blocks.pop() {
        let block = data.blocks.remove(&name).unwrap();
        out.blocks
            .push(convert_block(name, block, &mut blocks, &mut fieldsets));
    }

    while let Some(name) = fieldsets.pop() {
        let fieldset = data.fieldsets.remove(&name).unwrap();
        out.bitfields
            .push(convert_fieldset(name, fieldset, &mut enums));
    }

    while let Some(name) = enums.pop() {
        let enumm = data.enums.remove(&name).unwrap();
        out.enums.push(convert_enum(name, enumm));
    }

    Ok(out)
}

fn convert_block(
    name: String,
    block: cir::Block,
    blocks: &mut Vec<String>,
    fieldsets: &mut Vec<String>,
) -> ir::Block {
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
                blocks.push(block.block.clone());

                ir::block::FieldInner::Block(ir::block::field::Block {
                    block_name: block.block,
                })
            }
            cir::BlockItemInner::Register(reg) => {
                if let Some(fieldset) = reg.fieldset {
                    fieldsets.push(fieldset.clone());

                    ir::block::FieldInner::Bitfield(ir::block::field::Bitfield {
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
                        enumm: None
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

fn convert_fieldset(
    name: String,
    fieldset: cir::FieldSet,
    enums: &mut Vec<String>,
) -> ir::Bitfield {
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

        if let Some(enumm) = &field.enumm {
            enums.push(enumm.clone());
        }

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

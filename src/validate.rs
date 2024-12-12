use std::collections::HashSet;

use crate::ir::*;

struct ValidationCtx<'a> {
    blocks: HashSet<&'a str>,
    bitfields: HashSet<&'a str>,
    enums: HashSet<&'a str>,
}

impl<'a> ValidationCtx<'a> {
    fn new(module: &'a Module) -> Self {
        Self {
            blocks: module
                .blocks
                .iter()
                .map(|block| block.name.as_str())
                .collect::<HashSet<&str>>(),
            bitfields: module
                .bitfields
                .iter()
                .map(|bitfield| bitfield.name.as_str())
                .collect::<HashSet<&str>>(),
            enums: module
                .enums
                .iter()
                .map(|enumm| enumm.name.as_str())
                .collect::<HashSet<&str>>(),
        }
    }
}

pub fn validate_module(module: &Module) -> Vec<String> {
    let ctx = ValidationCtx::new(module);
    let mut errors = Vec::new();

    for block in &module.blocks {
        validate_block(block, &ctx, &mut errors);
    }

    for bitfield in &module.bitfields {
        validate_bitfield(bitfield, &ctx, &mut errors);
    }

    for enumm in &module.enums {
        validate_enum(enumm, &mut errors);
    }

    errors
}

fn validate_block(block: &Block, ctx: &ValidationCtx, errors: &mut Vec<String>) {
    if !is_valid_ident(&block.name) {
        errors.push(format!("block name {} is not a valid ident", block.name));
    }

    for field in &block.fields {
        if !is_valid_ident(&field.name) {
            errors.push(format!(
                "block {} field {} is not a valid ident",
                block.name, field.name
            ));
        }

        match &field.inner {
            block::FieldInner::Block(field2) => {
                if !ctx.blocks.contains(field2.block_name.as_str()) {
                    errors.push(format!(
                        "block {} field {} references non existent block {}",
                        block.name, field.name, field2.block_name
                    ));
                }
            }
            block::FieldInner::Bitfield(field2) => {
                if !ctx.bitfields.contains(field2.bitfield_name.as_str()) {
                    errors.push(format!(
                        "block {} field {} references non existent bitfield {}",
                        block.name, field.name, field2.bitfield_name
                    ));
                }
            }
            block::FieldInner::Simple(field2) => {
                if let Some(enumm) = &field2.enumm {
                    if !ctx.enums.contains(enumm.as_str()) {
                        errors.push(format!(
                            "block {} field {} references non existent enum {}",
                            block.name, field.name, enumm
                        ))
                    }
                }
            }
        }
    }
}

fn validate_bitfield(bitfield: &Bitfield, ctx: &ValidationCtx, errors: &mut Vec<String>) {
    if !is_valid_ident(&bitfield.name) {
        errors.push(format!(
            "bitfield name {} is not a valid ident",
            bitfield.name
        ));
    }

    for field in &bitfield.fields {
        if !is_valid_ident(&field.name) {
            errors.push(format!(
                "bitfield {} field {} is not a valid ident",
                bitfield.name, field.name
            ));
        }

        if field.bit_offset + field.bit_size > bitfield.bit_size {
            errors.push(format!(
                "bitfield {} field {} exceeds bitfield size",
                bitfield.name, field.name
            ));
        }

        if let Some(enumm) = &field.enumm {
            if !ctx.enums.contains(enumm.as_str()) {
                errors.push(format!(
                    "bitfield {} field {} references non existent enum {}",
                    bitfield.name, field.name, enumm
                ))
            }
        }
    }
}

fn validate_enum(enumm: &Enum, errors: &mut Vec<String>) {
    if !is_valid_ident(&enumm.name) {
        errors.push(format!("enum name {} is not a valid ident", enumm.name));
    }

    for variant in &enumm.variants {
        if !is_valid_ident(&variant.name) {
            errors.push(format!(
                "enum {} variant {} is not a valid ident",
                enumm.name, variant.name
            ));
        }

        if !can_fit(variant.value, enumm.bit_size) {
            errors.push(format!(
                "enum {} variant {} exceeds enum size",
                enumm.name, variant.name
            ))
        }
    }
}

fn can_fit(value: u64, bit_size: u32) -> bool {
    if value == 0 && bit_size > 0 {
        true
    } else {
        (value.ilog2() + 1) <= bit_size
    }
}

fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(c) = chars.next() else {
        return false;
    };

    if !c.is_ascii_alphabetic() && c != '_' && c != '-' {
        return false;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
            return false;
        }
    }

    true
}

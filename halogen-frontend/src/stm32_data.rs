use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result, ensure};
use chiptool::ir as cir;
use regex::Regex;
use stm32_data_serde::*;

use halogen_ir::ir::{self, MultiChip};

use crate::chiptool::convert_chiptool;
use crate::utils;
use crate::utils::rayon_prelude::*;

pub fn convert_multi_chips(
    root: impl AsRef<Path>,
    filter: Option<&Regex>,
) -> Result<ir::MultiChip> {
    let root = root.as_ref();

    let chips = list_chips(root)?;

    let chips = utils::into_maybe_par_iter(chips)
        .filter(|chip| filter.map(|filter| filter.is_match(chip)).unwrap_or(true))
        .map(|chip| -> Result<ir::Chip> {
            let mut chip = parse_chip(root, &chip)?;
            let core = extract_core(&mut chip)?;

            let imports = validate_and_extract_imports(&core)?
                .into_iter()
                .map(|(name, version)| ir::chip::Import {
                    name,
                    version: Some(version),
                })
                .collect();

            Ok(ir::Chip {
                name: chip.name,
                description: None,
                imports,
                peripherals: convert_peripherals(core.peripherals),
                stm32_ext: Some(ir::chip::Stm32Ext { cm_name: core.name }),
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let regs = utils::into_maybe_par_iter(&chips)
        .map(|chip| utils::into_maybe_par_iter(&chip.imports))
        .flatten()
        .collect::<HashSet<_>>();

    let modules = utils::into_maybe_par_iter(regs)
        .map(|import| -> Result<ir::Module> {
            let version = import.version.clone().unwrap();

            let regs = parse_registers(root, &import.name, &version)?;

            let module = convert_chiptool(import.name.clone(), Some(version), regs)?;
            Ok(module)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(MultiChip { chips, modules })
}

fn list_chips(root: &Path) -> Result<Vec<String>> {
    let mut chips = Vec::new();

    for file in fs::read_dir(root.join("chips"))? {
        let file = file?;
        let kind = file.file_type()?;
        if !kind.is_file() {
            continue;
        }

        let name = file.file_name();
        let Some(name) = name.to_str() else { continue };

        let Some(name) = name.strip_suffix(".json") else {
            continue;
        };

        chips.push(name.to_ascii_lowercase());
    }

    Ok(chips)
}

fn parse_chip(root: &Path, chip: &str) -> Result<Chip> {
    let path = root
        .join("chips")
        .join(format!("{}.json", chip.to_uppercase()));
    ensure!(path.exists(), "chip not found in data directory");

    let data = fs::read(path).context("failed to read chip file")?;
    let data = serde_json::from_slice(&data).context("failed to parse chip json")?;

    Ok(data)
}

fn parse_registers(root: &Path, name: &str, version: &str) -> Result<cir::IR> {
    let path = root
        .join("registers")
        .join(format!("{name}_{version}.json"));
    ensure!(path.exists(), "registers not found in data directory");

    let data = fs::read(path).context("failed to read registers file")?;
    let data = serde_json::from_slice(&data).context("failed to parse registers json")?;

    Ok(data)
}

fn convert_peripherals(peripherals: Vec<chip::core::Peripheral>) -> Vec<ir::chip::Peripheral> {
    let mut out = Vec::new();
    for peripheral in peripherals {
        let Some(regs) = peripheral.registers else {
            continue;
        };

        out.push(ir::chip::Peripheral {
            name: peripheral.name,
            description: None,
            module: regs.kind,
            address: peripheral.address as _,
            block_name: regs.block,
        });
    }

    out
}

fn extract_core(chip: &mut Chip) -> Result<chip::Core> {
    ensure!(
        chip.cores.len() == 1,
        "multicore devices are not yet supported"
    );

    Ok(chip.cores.pop().unwrap())
}

fn validate_and_extract_imports(core: &chip::Core) -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    for peripheral in core.peripherals.iter() {
        let Some(regs) = &peripheral.registers else {
            continue;
        };

        // First check versions for multiple versions
        match out.entry(regs.kind.clone()) {
            Entry::Occupied(entry) => ensure!(
                entry.get() == &regs.version,
                "multiple registers versions used"
            ),
            Entry::Vacant(entry) => {
                entry.insert(regs.version.clone());
            }
        }
    }

    Ok(out)
}

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result, ensure};
use chiptool::ir as cir;
use stm32_data_serde::*;

use halogen_ir::ir::{self, MultiChip};

use crate::chiptool::convert_chiptool;
use crate::utils;
use crate::utils::rayon_prelude::*;

pub fn convert_multi_chips(root: impl AsRef<Path>, chips: &[String]) -> Result<ir::MultiChip> {
    let root = root.as_ref();

    let chips = utils::into_maybe_par_iter(chips)
        .map(|chip| -> Result<ir::Chip> {
            let mut chip = parse_chip(root, chip)?;
            let core = extract_core(&mut chip)?;

            let imports = validate_and_extract_imports(&core)?
                .into_iter()
                .map(|(name, version)| ir::chip::Import { name, version })
                .collect();

            Ok(ir::Chip {
                name: chip.name,
                description: None,
                imports,
                peripherals: convert_peripherals(core.peripherals),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let regs = utils::into_maybe_par_iter(&chips)
        .map(|chip| utils::into_maybe_par_iter(&chip.imports))
        .flatten()
        .collect::<HashSet<_>>();

    let modules = utils::into_maybe_par_iter(regs)
        .map(|import| -> Result<ir::Module> {
            let regs = parse_registers(root, &import.name, &import.version)?;

            let module = convert_chiptool(import.name.clone(), Some(import.version.clone()), regs)?;
            Ok(module)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(MultiChip { chips, modules })
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
            block: regs.block,
            address: peripheral.address as _,
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

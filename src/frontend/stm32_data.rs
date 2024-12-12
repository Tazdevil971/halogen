use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{ensure, Context as _, Result};
use chiptool::ir as cir;
use stm32_data_serde::*;

use crate::frontend::chiptool::convert_chiptool;
use crate::ir;
use crate::utils::into_maybe_par_iter;
use crate::utils::rayon_prelude::*;

pub fn convert_multi_chips(root: &Path, chips: &[String]) -> Result<ir::MultiChip> {
    let mut out = ir::MultiChip {
        chips: Vec::new(),
        modules: Vec::new(),
    };

    out.chips = into_maybe_par_iter(chips)
        .map(|chip| -> Result<ir::Chip> {
            let mut chip = parse_chip(root, chip)?;
            let core = extract_core(&mut chip)?;

            let imports = validate_and_extract_imports(&core)?;

            let mut modules = Vec::new();
            for (name, version) in imports {
                modules.push(ir::ModuleOrImport::Import(ir::Import { name, version }));
            }

            Ok(ir::Chip {
                name: chip.name,
                description: None,
                modules,
                peripherals: convert_peripherals(core.peripherals),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let regs = into_maybe_par_iter(&out.chips)
        .map(|chip| {
            chip.modules.par_iter().filter_map(|module| match module {
                ir::ModuleOrImport::Module(_) => None,
                ir::ModuleOrImport::Import(import) => Some((&import.name, &import.version)),
            })
        })
        .flatten()
        .collect::<HashSet<_>>();

    out.modules = into_maybe_par_iter(regs)
        .map(|(name, version)| -> Result<ir::Module> {
            let regs = parse_registers(root, &name, &version)?;

            let module = convert_chiptool(name.into(), Some(version.into()), regs)?;
            Ok(module)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(out)
}

pub fn convert_chip(root: &Path, chip: &str) -> Result<ir::Chip> {
    let mut chip = parse_chip(root, chip)?;
    let core = extract_core(&mut chip)?;

    let regs = validate_and_extract_imports(&core)?;

    let mut out = ir::Chip {
        name: chip.name,
        description: None,
        modules: Vec::new(),
        peripherals: convert_peripherals(core.peripherals),
    };

    out.modules = into_maybe_par_iter(regs)
        .map(|(name, version)| -> Result<ir::ModuleOrImport> {
            let regs = parse_registers(root, &name, &version)?;

            let module = convert_chiptool(name, Some(version), regs)?;
            Ok(ir::ModuleOrImport::Module(module))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(out)
}

fn parse_chip(root: &Path, chip: &str) -> Result<Chip> {
    let path = root
        .join("chips")
        .join(format!("{}.json", chip.to_uppercase()));
    ensure!(path.exists(), "chip not found in data directory");

    let data = std::fs::read_to_string(path).context("failed to read chip file")?;
    let data = serde_json::from_str(&data).context("failed to parse chip json")?;

    Ok(data)
}

fn parse_registers(root: &Path, name: &str, version: &str) -> Result<cir::IR> {
    let path = root
        .join("registers")
        .join(format!("{name}_{version}.json"));
    ensure!(path.exists(), "registers not found in data directory");

    let data = std::fs::read_to_string(path).context("failed to read registers file")?;
    let data = serde_json::from_str(&data).context("failed to parse registers json")?;

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

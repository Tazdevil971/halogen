use std::path::{Path, PathBuf};
use std::{fs, io};

use heck::ToSnakeCase as _;
use serde::{Deserialize, Serialize};

use crate::utils::rayon_prelude::*;
use crate::{ir, utils};

pub fn load_single_file(path: impl AsRef<Path>) -> io::Result<ir::MultiChip> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

pub fn dump_single_file(path: impl AsRef<Path>, ir: &ir::MultiChip) -> io::Result<()> {
    let data = serde_json::to_vec_pretty(ir)?;
    fs::write(path, &data)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Index {
    chips: Vec<PathBuf>,
    modules: Vec<PathBuf>,
}

pub fn load_multi_file(path: impl AsRef<Path>) -> io::Result<ir::MultiChip> {
    let data = fs::read(path)?;
    let index = serde_json::from_slice::<Index>(&data)?;

    let (chips, modules) = utils::maybe_par_join(
        || {
            utils::into_maybe_par_iter(index.chips)
                .map(|path| {
                    let data = fs::read(path)?;
                    let chip = serde_json::from_slice::<ir::Chip>(&data)?;

                    Ok(chip)
                })
                .collect::<io::Result<Vec<_>>>()
        },
        || {
            utils::into_maybe_par_iter(index.modules)
                .map(|path| {
                    let data = fs::read(path)?;
                    let chip = serde_json::from_slice::<ir::Module>(&data)?;

                    Ok(chip)
                })
                .collect::<io::Result<Vec<_>>>()
        },
    );

    let chips = chips?;
    let modules = modules?;

    Ok(ir::MultiChip { chips, modules })
}

pub fn dump_multi_file(root: impl AsRef<Path>, ir: &ir::MultiChip) -> io::Result<()> {
    let root = root.as_ref();
    let chips_path = root.join("chips");
    let modules_path = root.join("modules");

    utils::create_dir_if_not_exist(&chips_path)?;
    utils::create_dir_if_not_exist(&modules_path)?;

    let (chips, modules) = utils::maybe_par_join(
        || {
            utils::into_maybe_par_iter(&ir.chips)
                .map(|chip| {
                    let path = chips_path.join(format!("{}.json", chip.name.to_snake_case()));

                    let data = serde_json::to_vec_pretty(&chip)?;
                    fs::write(&path, &data)?;

                    Ok(path)
                })
                .collect::<io::Result<Vec<_>>>()
        },
        || {
            utils::into_maybe_par_iter(&ir.modules)
                .map(|module| {
                    let path = modules_path.join(format!("{}.json", module.name.to_snake_case()));

                    let data = serde_json::to_vec_pretty(&module)?;
                    fs::write(&path, &data)?;

                    Ok(path)
                })
                .collect::<io::Result<Vec<_>>>()
        },
    );

    let chips = chips?;
    let modules = modules?;

    let index = Index { chips, modules };

    let data = serde_json::to_vec_pretty(&index)?;
    fs::write(root.join("index.json"), &data)?;

    Ok(())
}

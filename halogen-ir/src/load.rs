use std::path::{Path, PathBuf};
use std::{fs, io};

use heck::ToSnakeCase as _;
use serde::{Deserialize, Serialize};

use crate::utils::rayon_prelude::*;
use crate::{ir, utils};

fn load_json_file<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> io::Result<T> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

fn load_json_reader<T: for<'a> Deserialize<'a>>(reader: impl io::Read) -> io::Result<T> {
    Ok(serde_json::from_reader(reader)?)
}

fn dump_json_file(path: impl AsRef<Path>, value: &impl Serialize) -> io::Result<()> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(path, &data)?;
    Ok(())
}

fn dump_json_writer(writer: impl io::Write, value: &impl Serialize) -> io::Result<()> {
    Ok(serde_json::to_writer_pretty(writer, value)?)
}

/// Load the IR from a reader.
pub fn load_reader(reader: impl io::Read) -> io::Result<ir::MultiChip> {
    load_json_reader(reader)
}

/// Load the IR from a writer.
pub fn dump_writer(writer: impl io::Write, ir: &ir::MultiChip) -> io::Result<()> {
    dump_json_writer(writer, ir)
}

/// Load the IR from a single file.
pub fn load_single_file(path: impl AsRef<Path>) -> io::Result<ir::MultiChip> {
    load_json_file(path)
}

/// Dump the IR to a single file.
pub fn dump_single_file(path: impl AsRef<Path>, ir: &ir::MultiChip) -> io::Result<()> {
    dump_json_file(path, ir)
}

#[derive(Debug, Serialize, Deserialize)]
struct Index {
    chips: Vec<PathBuf>,
    modules: Vec<PathBuf>,
}

/// Load the IR from multiple files, the path points to the index
pub fn load_multi_file(path: impl AsRef<Path>) -> io::Result<ir::MultiChip> {
    let index = load_json_file::<Index>(path)?;

    let (chips, modules) = utils::maybe_par_join(
        || {
            utils::into_maybe_par_iter(index.chips)
                .map(load_json_file)
                .collect::<io::Result<Vec<_>>>()
        },
        || {
            utils::into_maybe_par_iter(index.modules)
                .map(load_json_file)
                .collect::<io::Result<Vec<_>>>()
        },
    );

    let chips = chips?;
    let modules = modules?;

    Ok(ir::MultiChip { chips, modules })
}

/// Dump the IR to multiple files, the path points to the root directory.
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
                    dump_json_file(&path, chip)?;

                    Ok(path)
                })
                .collect::<io::Result<Vec<_>>>()
        },
        || {
            utils::into_maybe_par_iter(&ir.modules)
                .map(|module| {
                    let path = modules_path.join(format!("{}.json", module.name.to_snake_case()));
                    dump_json_file(&path, module)?;

                    Ok(path)
                })
                .collect::<io::Result<Vec<_>>>()
        },
    );

    let chips = chips?;
    let modules = modules?;

    dump_json_file(root.join("index.json"), &Index { chips, modules })?;

    Ok(())
}

use std::ffi::OsStr;
use std::io;
use std::path::Path;

use anyhow::Result;

use halogen_ir::ir;
use halogen_ir::load::*;

pub mod gen_rust;
pub mod stm32_data_convert;

#[derive(Debug, clap::Parser)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmds,
}

#[derive(Debug, clap::Subcommand)]
pub enum Cmds {
    Stm32DataConvert(stm32_data_convert::args::Args),
    GenRust(gen_rust::args::Args),
}

fn load_ir(path: impl AsRef<Path>) -> Result<ir::MultiChip> {
    if path.as_ref() == OsStr::new("-") {
        Ok(load_reader(io::stdin().lock())?)
    } else {
        Ok(load_multi_file(path)?)
    }
}

fn dump_ir(path: impl AsRef<Path>, ir: &ir::MultiChip, multi: bool) -> Result<()> {
    if path.as_ref() == OsStr::new("-") {
        dump_writer(io::stdout().lock(), ir)?;
    } else if multi {
        dump_multi_file(path, ir)?;
    } else {
        dump_single_file(path, ir)?;
    }

    Ok(())
}

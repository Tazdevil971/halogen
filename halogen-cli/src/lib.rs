use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    cmd: Cmds,
}

#[derive(Debug, clap::Subcommand)]
pub enum Cmds {
    Stm32DataConvert(Stm32DataConvertArgs),
    GenRust { input: PathBuf, output: PathBuf },
}

#[derive(Debug, clap::Args)]
pub struct Stm32DataConvertArgs {
    /// Root of the stm32-data folder
    input: PathBuf,
    output: PathBuf,
    filter: String,
    #[arg(value_enum)]
    output_format: IrFormat,
}

#[derive(Debug, clap::Args)]
pub struct GenRustArgs {
    input: PathBuf,
    output: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum IrFormat {
    /// All of the IR is contained in a single file
    SingleFile,
    /// The IR is split across multiple files in a directory
    MultiFile,
}

use std::path::PathBuf;

use anyhow::Result;

use crate::dump_ir;
use halogen_frontend::stm32_data;

pub mod args {
    use super::*;

    #[derive(Debug, clap::Args)]
    pub struct Args {
        /// Root of the stm32-data folder
        #[arg(short, long)]
        pub input: PathBuf,
        /// Output path of the IR
        #[arg(short, long)]
        pub output: PathBuf,
        /// Regex filtering which boards will actually be included in the generated IR.
        #[arg(long)]
        pub filter: Option<regex::Regex>,
        /// Output using the multi-file IR format.
        #[arg(long, default_value_t = false)]
        pub multi: bool,
    }
}

pub fn run(args: &args::Args) -> Result<()> {
    // First generate IR
    let ir = stm32_data::convert_multi_chips(&args.input, args.filter.as_ref())?;

    // TODO: Do a bit of postprocess on the IR (normalization, dead stuff elimination, validation)

    // Finally dump IR
    dump_ir(&args.output, &ir, args.multi)?;

    Ok(())
}

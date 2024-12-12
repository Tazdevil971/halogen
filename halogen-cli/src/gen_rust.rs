use std::path::PathBuf;

use anyhow::Result;

use crate::load_ir;
use halogen_backend::rust;

pub mod args {
    use super::*;

    #[derive(Debug, clap::Args)]
    pub struct Args {
        /// Input halogen IR.
        #[arg(short, long)]
        pub input: PathBuf,
        /// Output folder for the generated rust bindings
        #[arg(short, long)]
        pub output: PathBuf,
        /// Where to locate the utility module
        #[arg(long, value_enum, default_value_t = Utils::Embed)]
        pub utils: Utils,
        #[arg(long, value_enum, default_value_t = Format::Rustfmt)]
        pub format: Format,
        /// Path used for auxiliary core definitions (Cortex-M)
        #[arg(long)]
        pub core_path: Option<String>,
        /// Do not generate chips.rs file
        #[arg(long)]
        pub dont_gen_chips: bool,
        /// Do not generate list.rs file
        #[arg(long)]
        pub dont_gen_list: bool,
    }

    #[derive(Debug, Clone, Copy, clap::ValueEnum)]
    pub enum Utils {
        /// Import it from super module
        Super,
        /// Embed it into the top level generated module
        Embed,
        /// completely ignore utils module
        None,
    }

    #[derive(Debug, Clone, Copy, clap::ValueEnum)]
    pub enum Format {
        /// Use rustfmt as the formatter
        Rustfmt,
        /// Do not use any formatter
        None,
    }
}

pub fn run(args: &args::Args) -> Result<()> {
    log::info!("Loading IR...");
    let ir = load_ir(&args.input)?;

    log::info!("Generating bindings...");
    let ctx = rust::GenCtx::new();
    ctx.gen_multi_chip(
        &ir,
        &args.output,
        rust::GenMultiChipSettings {
            utils: match args.utils {
                args::Utils::Embed => rust::Utils::Embed,
                args::Utils::Super => rust::Utils::Super,
                args::Utils::None => rust::Utils::None,
            },
            format: match args.format {
                args::Format::Rustfmt => rust::Format::Rustfmt,
                args::Format::None => rust::Format::None,
            },
            core_path: args.core_path.as_deref(),
            gen_chips: !args.dont_gen_chips,
            gen_list: !args.dont_gen_list,
        },
    )?;

    log::info!("Generation finished!");
    Ok(())
}

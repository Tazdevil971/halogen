use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use halogen_cli::*;

fn main() -> ExitCode {
    env_logger::init();

    let args = Args::parse();

    match try_main(&args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {}", err);
            ExitCode::FAILURE
        }
    }
}

fn try_main(args: &Args) -> Result<()> {
    match &args.cmd {
        Cmds::GenRust(args) => gen_rust::run(&args),
        Cmds::Stm32DataConvert(args) => stm32_data_convert::run(&args),
    }
}

use clap::{crate_description, value_parser};
use std::path::PathBuf;

use clap::{ArgAction, Command, arg, crate_version};

pub fn cli() -> Command {
    Command::new("impala")
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            arg!(--config <path>)
                .short('c')
                .required(false)
                .help("config file path. Default ~/.config/bluetui/config.toml")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--ascii)
                .required(false)
                .help("Ascii display")
                .action(ArgAction::SetTrue),
        )
}

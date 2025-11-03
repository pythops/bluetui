use std::path::PathBuf;

use clap::{Command, arg, crate_description, crate_name, crate_version, value_parser};

pub fn cli() -> Command {
    Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            arg!(--config <config>)
                .short('c')
                .id("config")
                .required(false)
                .help("Config file path")
                .value_parser(value_parser!(PathBuf)),
        )
}

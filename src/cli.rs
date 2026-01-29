use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    pub config_path: Option<PathBuf>,
}

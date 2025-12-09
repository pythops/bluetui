use crate::app::AppResult;
use anyhow::Context;
use bluer::Address;
use clap::crate_name;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn read_favorite_devices_from_disk() -> AppResult<Vec<Address>> {
    let data_dir = dirs::data_dir()
        .context("unable to find data_dir")?
        .join(crate_name!());

    let file = tokio::fs::File::open(data_dir.join("favorites.txt"))
        .await
        .context("unable to open favorites file")?;

    let mut lines = BufReader::new(file).lines();

    let mut favorite_devices = Vec::new();

    while let Some(line) = lines.next_line().await? {
        if let Ok(addr) = Address::from_str(&line) {
            favorite_devices.push(addr);
        }
    }

    Ok(favorite_devices)
}

pub fn save_favorite_devices_to_disk(favorite_devices: &[Address]) -> AppResult<()> {
    let data_dir = dirs::data_dir()
        .context("unable to find data_dir")?
        .join(crate_name!());

    let file_path = data_dir.join("favorites.txt");

    let contents = favorite_devices
        .iter()
        .map(Address::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)
            .context("unable to create parent dir(s) to favorites file")?;
    }

    std::fs::write(file_path, contents).context("error writing favorites file")?;

    Ok(())
}

use std::sync::{Arc, atomic::AtomicBool};

use bluer::{Adapter, Address, Session};

use bluer::Device as BTDevice;

use crate::app::AppResult;

#[derive(Debug, Clone)]
pub struct Controller {
    pub adapter: Arc<Adapter>,
    pub name: String,
    pub alias: String,
    pub is_powered: bool,
    pub is_pairable: bool,
    pub is_discoverable: bool,
    pub is_scanning: Arc<AtomicBool>,
    pub paired_devices: Vec<Device>,
    pub new_devices: Vec<Device>,
}

#[derive(Debug, Clone)]
pub struct Device {
    device: BTDevice,
    pub addr: Address,
    pub icon: &'static str,
    pub alias: String,
    pub is_paired: bool,
    pub is_favorite: bool,
    pub is_trusted: bool,
    pub is_connected: bool,
    pub battery_percentage: Option<u8>,

    /// user has requested connect/disconnect and (bluer) hasn't yet performed the action
    /// when true, a spinner is shown in the ui
    pub is_busy: Arc<AtomicBool>,
}

impl Device {
    pub async fn set_alias(&self, alias: String) -> AppResult<()> {
        self.device.set_alias(alias).await?;
        Ok(())
    }

    // https://specifications.freedesktop.org/icon-naming/latest/
    pub fn get_icon(name: &str) -> &'static str {
        match name {
            "audio-card" => "󰓃 ",
            "audio-input-microphone" => " ",
            "audio-headphones" | "audio-headset" => "󰋋 ",
            "battery" => "󰂀 ",
            "camera-photo" => "󰻛 ",
            "computer" => " ",
            "input-keyboard" => "󰌌 ",
            "input-mouse" => "󰍽 ",
            "input-gaming" => "󰊴 ",
            "phone" => "󰏲 ",
            _ => "󰾰 ",
        }
    }
}

impl Controller {
    pub async fn get_all(
        session: Arc<Session>,
        favorite_devices: &[Address],
    ) -> AppResult<Vec<Controller>> {
        let mut controllers: Vec<Controller> = Vec::new();

        // let session = bluer::Session::new().await?;
        let adapter_names = session.adapter_names().await?;
        for adapter_name in adapter_names {
            if let Ok(adapter) = session.adapter(&adapter_name) {
                let name = adapter.name().to_owned();
                let alias = adapter.alias().await?;
                let is_powered = adapter.is_powered().await?;
                let is_pairable = adapter.is_pairable().await?;
                let is_discoverable = adapter.is_discoverable().await?;
                let is_scanning = adapter.is_discovering().await?;

                let (paired_devices, new_devices) =
                    Controller::get_all_devices(&adapter, favorite_devices).await?;

                let controller = Controller {
                    adapter: Arc::new(adapter),
                    name,
                    alias,
                    is_powered,
                    is_pairable,
                    is_discoverable,
                    is_scanning: Arc::new(AtomicBool::new(is_scanning)),
                    paired_devices,
                    new_devices,
                };

                controllers.push(controller);
            }
        }

        Ok(controllers)
    }

    pub async fn get_all_devices(
        adapter: &Adapter,
        favorite_devices: &[Address],
    ) -> AppResult<(Vec<Device>, Vec<Device>)> {
        let mut paired_devices: Vec<Device> = Vec::new();
        let mut new_devices: Vec<Device> = Vec::new();
        let mut devices_without_aliases: Vec<Device> = Vec::new();

        let connected_devices_addresses = adapter.device_addresses().await?;
        for addr in connected_devices_addresses {
            let device = adapter.device(addr)?;

            let alias = device.alias().await?;
            let icon = Device::get_icon(device.icon().await?.unwrap_or("-".to_string()).as_str());
            let is_paired = device.is_paired().await?;
            let is_trusted = device.is_trusted().await?;
            let is_connected = device.is_connected().await?;
            let is_favorite = favorite_devices.contains(&addr);
            let battery_percentage = device.battery_percentage().await?;

            let dev = Device {
                device,
                addr,
                alias,
                icon,
                is_paired,
                is_trusted,
                is_connected,
                is_favorite,
                battery_percentage,
                is_busy: Arc::new(AtomicBool::new(false)),
            };

            if dev.is_paired {
                paired_devices.push(dev);
            } else {
                match is_mac_addr(&dev.alias) {
                    // most device names without aliases may default to their mac addresses, but we should not
                    // assume that to be 100% the case
                    true => devices_without_aliases.push(dev),
                    false => new_devices.push(dev),
                }
            }
        }

        paired_devices.sort_by_key(|i| (!i.is_favorite, i.addr));
        new_devices.sort_by(|a, b| a.alias.cmp(&b.alias));
        devices_without_aliases.sort_by_key(|i| i.addr);
        new_devices.extend(devices_without_aliases);

        Ok((paired_devices, new_devices))
    }
}

fn is_mac_addr(s: &str) -> bool {
    if s.len() != 17 {
        return false;
    }
    let mut chars = s.chars();
    for _ in 0..5 {
        // Matches [A-Fa-f0-9][A-Fa-f0-9]-
        if !(matches!(chars.next(), Some(c) if c.is_ascii_hexdigit())
            && matches!(chars.next(), Some(c) if c.is_ascii_hexdigit())
            && matches!(chars.next(), Some('-')))
        {
            return false;
        }
    }
    // Matches [A-Fa-f0-9][A-Fa-f0-9]$
    matches!(chars.next(), Some(c) if c.is_ascii_hexdigit())
        && matches!(chars.next(), Some(c) if c.is_ascii_hexdigit())
        && chars.next().is_none()
}

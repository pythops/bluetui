use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

use async_channel::Receiver;
use bluer::{
    agent::{ReqError, ReqResult, RequestConfirmation},
    Adapter, Address, Session,
};
use tokio::sync::oneshot;

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
    pub addr: Address,
    pub alias: String,
    pub is_paired: bool,
    pub is_trusted: bool,
    pub is_connected: bool,
    pub battery_percentage: Option<u8>,
}

// https://specifications.freedesktop.org/icon-naming-spec/icon-naming-spec-latest.html
impl Device {
    pub fn get_icon(name: &str) -> String {
        match name {
            "audio-card" => String::from("󰓃"),
            "audio-input-microphone" => String::from(""),
            "audio-headphones" => String::from("󰋋"),
            "battery" => String::from("󰂀"),
            "camera-photo" => String::from("󰻛"),
            "computer" => String::from(""),
            "input-keyboard" => String::from("󰌌"),
            "input-mouse" => String::from("󰍽"),
            "phone" => String::from("󰏲"),
            _ => String::from(" "),
        }
    }
}

impl Controller {
    pub async fn get_all(session: Arc<Session>) -> AppResult<Vec<Controller>> {
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

                let (paired_devices, new_devices) = Controller::get_all_devices(&adapter).await?;

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

    pub async fn get_all_devices(adapter: &Adapter) -> AppResult<(Vec<Device>, Vec<Device>)> {
        let mut paired_devices: Vec<Device> = Vec::new();
        let mut new_devices: Vec<Device> = Vec::new();

        let connected_devices_addresses = adapter.device_addresses().await?;
        for addr in connected_devices_addresses {
            let device = adapter.device(addr)?;

            let alias = device.alias().await?;
            let icon = Device::get_icon(device.icon().await?.unwrap_or("-".to_string()).as_str());
            let is_paired = device.is_paired().await?;
            let is_trusted = device.is_trusted().await?;
            let is_connected = device.is_connected().await?;
            let battery_percentage = device.battery_percentage().await?;

            let device = Device {
                addr,
                alias: format!("{} {}", icon, alias),
                is_paired,
                is_trusted,
                is_connected,
                battery_percentage,
            };

            if device.is_paired {
                paired_devices.push(device);
            } else {
                new_devices.push(device);
            }
        }

        paired_devices.sort_by_key(|i| i.addr);
        new_devices.sort_by_key(|i| i.addr);
        Ok((paired_devices, new_devices))
    }
}

pub async fn request_confirmation(
    req: RequestConfirmation,
    display_confirmation_popup: Arc<AtomicBool>,
    rx: Receiver<bool>,
    sender: Sender<String>,
) -> ReqResult<()> {
    display_confirmation_popup.store(true, std::sync::atomic::Ordering::Relaxed);

    sender
        .send(format!(
            "Is passkey \"{:06}\" correct for device {} on {}?",
            req.passkey, &req.device, &req.adapter
        ))
        .unwrap();

    // request cancel
    let (_done_tx, done_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        if done_rx.await.is_err() {
            display_confirmation_popup.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    });
    match rx.recv().await {
        Ok(v) => {
            // false: reject the confirmation
            if !v {
                return Err(ReqError::Rejected);
            }
        }
        Err(_) => return Err(ReqError::Rejected),
    }

    Ok(())
}

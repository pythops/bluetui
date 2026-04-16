use std::collections::{HashMap, HashSet};
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Duration;

use bluer::{Adapter, Address, Session, Uuid};
use tokio::sync::RwLock;

use bluer::Device as BTDevice;

use crate::app::AppResult;

pub type BatteryCache = Arc<RwLock<HashMap<Address, Option<u8>>>>;
pub type PendingFetches = Arc<RwLock<HashSet<Address>>>;

// GATT UUIDs for Battery Service
const BATTERY_SERVICE_UUID: Uuid = Uuid::from_u128(0x0000180f_0000_1000_8000_00805f9b34fb);
const BATTERY_LEVEL_CHAR_UUID: Uuid = Uuid::from_u128(0x00002a19_0000_1000_8000_00805f9b34fb);
const USER_DESCRIPTION_DESC_UUID: Uuid = Uuid::from_u128(0x00002901_0000_1000_8000_00805f9b34fb);

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
    pub battery_percentage_peripheral: Option<u8>,
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

    /// Check if a GATT characteristic has a User Description descriptor.
    /// This is used to identify peripheral battery characteristics in split keyboards.
    async fn has_user_description(descriptors: &[bluer::gatt::remote::Descriptor]) -> bool {
        for descriptor in descriptors {
            if descriptor
                .uuid()
                .await
                .is_ok_and(|u| u == USER_DESCRIPTION_DESC_UUID)
            {
                return true;
            }
        }
        false
    }

    /// Read and validate battery level from a GATT characteristic.
    /// Returns None if the value is invalid or out of range (0-100).
    async fn read_and_validate_battery_level(
        characteristic: &bluer::gatt::remote::Characteristic,
    ) -> Option<u8> {
        let value = characteristic.read().await.ok()?;
        let &level = value.first()?;

        // Validate battery level is in valid range (0-100)
        (level <= 100).then_some(level)
    }

    /// Find peripheral battery level from GATT Battery Service.
    /// Returns the first battery characteristic that has a User Description descriptor.
    async fn find_peripheral_battery_in_gatt(device: &BTDevice) -> Option<u8> {
        let services = device.services().await.ok()?;

        for service in services {
            // Skip non-battery services
            if service.uuid().await.ok()? != BATTERY_SERVICE_UUID {
                continue;
            }

            let characteristics = service.characteristics().await.ok()?;

            for characteristic in characteristics {
                // Skip non-battery-level characteristics
                if characteristic.uuid().await.ok()? != BATTERY_LEVEL_CHAR_UUID {
                    continue;
                }

                // Check for User Description descriptor (identifies peripheral battery)
                let descriptors = characteristic.descriptors().await.ok()?;
                if !Device::has_user_description(&descriptors).await {
                    continue;
                }

                // Found the peripheral battery - read and validate it
                let level = Device::read_and_validate_battery_level(&characteristic).await;
                if level.is_some() {
                    return level;
                }
            }
        }

        None
    }

    /// Read peripheral battery level from GATT Battery Service with a timeout.
    ///
    /// For devices with multiple batteries (e.g., split keyboards), the central battery
    /// is provided by the org.bluez.Battery1 interface (via `device.battery_percentage()`),
    /// while the peripheral battery is exposed through GATT Battery Service characteristics
    /// that have a User Description descriptor.
    ///
    /// Returns the battery percentage for the first characteristic with User Description.
    /// Times out after 5 seconds to prevent hanging.
    async fn get_peripheral_battery(device: &BTDevice) -> Option<u8> {
        const TIMEOUT_DURATION: Duration = Duration::from_secs(5);

        // Only try GATT if device is connected (required for service resolution)
        if !device.is_connected().await.unwrap_or(false) {
            return None;
        }

        // Wrap GATT operation in timeout to prevent UI hanging
        let fetch_task = Device::find_peripheral_battery_in_gatt(device);
        let timeout_result = tokio::time::timeout(TIMEOUT_DURATION, fetch_task).await;

        // Return None if timeout occurred, otherwise return the result
        timeout_result.ok().flatten()
    }

    /// Update cache with a new battery level.
    async fn update_battery_cache(cache: &BatteryCache, device_addr: Address, battery_level: u8) {
        let mut cache_guard = cache.write().await;
        cache_guard.insert(device_addr, Some(battery_level));
    }

    /// Mark a device as no longer having a pending fetch operation.
    async fn clear_pending_fetch(pending_fetches: &PendingFetches, device_addr: Address) {
        let mut pending_guard = pending_fetches.write().await;
        pending_guard.remove(&device_addr);
    }

    /// Spawn a background task to fetch peripheral battery and update cache.
    ///
    /// This function is non-blocking and won't hang the UI. It automatically:
    /// 1. Fetches the battery level with a timeout
    /// 2. Updates the cache if successful
    /// 3. Removes the device from the pending set when complete
    fn spawn_peripheral_battery_fetch(
        device: BTDevice,
        device_addr: Address,
        cache: BatteryCache,
        pending_fetches: PendingFetches,
    ) {
        tokio::spawn(async move {
            // Attempt to fetch battery level (with 5-second timeout)
            if let Some(battery_level) = Device::get_peripheral_battery(&device).await {
                Device::update_battery_cache(&cache, device_addr, battery_level).await;
            }

            // Always clear pending status, whether fetch succeeded or failed
            Device::clear_pending_fetch(&pending_fetches, device_addr).await;
        });
    }
}

impl Controller {
    /// Read peripheral battery from cache and invalidate it for the next refresh cycle.
    ///
    /// This ensures battery levels are constantly refreshed (every 1 second) like
    /// other device properties in the UI.
    async fn read_and_invalidate_battery_cache(
        cache: &BatteryCache,
        device_addr: Address,
    ) -> Option<u8> {
        let mut cache_guard = cache.write().await;
        cache_guard.remove(&device_addr).flatten()
    }

    /// Check if a fetch operation is already pending for this device.
    async fn is_fetch_pending(pending_fetches: &PendingFetches, device_addr: Address) -> bool {
        let pending_guard = pending_fetches.read().await;
        pending_guard.contains(&device_addr)
    }

    /// Mark a device as having a pending fetch operation.
    async fn mark_fetch_pending(pending_fetches: &PendingFetches, device_addr: Address) {
        let mut pending_guard = pending_fetches.write().await;
        pending_guard.insert(device_addr);
    }

    /// Fetch peripheral battery in the background if not already pending.
    ///
    /// This function:
    /// 1. Checks if a fetch is already in progress
    /// 2. If not, marks the device as pending and spawns a background task
    /// 3. The background task will update the cache when complete
    async fn maybe_spawn_battery_fetch(
        device: &BTDevice,
        device_addr: Address,
        is_connected: bool,
        battery_cache: &BatteryCache,
        pending_fetches: &PendingFetches,
    ) {
        // Only fetch for connected devices
        if !is_connected {
            return;
        }

        // Don't spawn duplicate fetches
        if Controller::is_fetch_pending(pending_fetches, device_addr).await {
            return;
        }

        // Mark as pending and spawn background task
        Controller::mark_fetch_pending(pending_fetches, device_addr).await;
        Device::spawn_peripheral_battery_fetch(
            device.clone(),
            device_addr,
            battery_cache.clone(),
            pending_fetches.clone(),
        );
    }

    pub async fn get_all(
        session: Arc<Session>,
        favorite_devices: &[Address],
        battery_cache: BatteryCache,
        pending_fetches: PendingFetches,
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

                let (paired_devices, new_devices) = Controller::get_all_devices(
                    &adapter,
                    favorite_devices,
                    battery_cache.clone(),
                    pending_fetches.clone(),
                )
                .await?;

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
        battery_cache: BatteryCache,
        pending_fetches: PendingFetches,
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

            // Get peripheral battery from cache (and invalidate for next refresh)
            let battery_percentage_peripheral =
                Controller::read_and_invalidate_battery_cache(&battery_cache, addr).await;

            // Spawn background fetch for next cycle (if not already pending)
            Controller::maybe_spawn_battery_fetch(
                &device,
                addr,
                is_connected,
                &battery_cache,
                &pending_fetches,
            )
            .await;

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
                battery_percentage_peripheral,
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

use toml;

use dirs;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_toggle_scanning")]
    pub toggle_scanning: char,

    #[serde(default)]
    pub adapter: Adapter,

    #[serde(default)]
    pub paired_device: PairedDevice,

    #[serde(default)]
    pub new_device: NewDevice,
}

#[derive(Deserialize, Debug)]
pub struct Adapter {
    #[serde(default = "default_toggle_adapter_pairing")]
    pub toggle_pairing: char,

    #[serde(default = "default_toggle_adapter_power")]
    pub toggle_power: char,

    #[serde(default = "default_toggle_adapter_discovery")]
    pub toggle_discovery: char,
}

impl Default for Adapter {
    fn default() -> Self {
        Self {
            toggle_pairing: 'p',
            toggle_power: 'o',
            toggle_discovery: 'd',
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PairedDevice {
    #[serde(default = "default_unpair_device")]
    pub unpair: char,

    #[serde(default = "default_toggle_device_connection")]
    pub toggle_connect: char,

    #[serde(default = "default_toggle_device_trust")]
    pub toggle_trust: char,

    #[serde(default = "default_set_new_name")]
    pub rename: char,
}

impl Default for PairedDevice {
    fn default() -> Self {
        Self {
            unpair: 'u',
            toggle_connect: ' ',
            toggle_trust: 't',
            rename: 'e',
        }
    }
}

fn default_set_new_name() -> char {
    'e'
}

#[derive(Deserialize, Debug)]
pub struct NewDevice {
    #[serde(default = "default_pair_new_device")]
    pub pair: char,
}

impl Default for NewDevice {
    fn default() -> Self {
        Self { pair: 'p' }
    }
}

fn default_toggle_scanning() -> char {
    's'
}

fn default_toggle_adapter_pairing() -> char {
    'p'
}

fn default_toggle_adapter_power() -> char {
    'o'
}

fn default_toggle_adapter_discovery() -> char {
    'd'
}

fn default_unpair_device() -> char {
    'u'
}

fn default_toggle_device_connection() -> char {
    ' '
}

fn default_toggle_device_trust() -> char {
    't'
}

fn default_pair_new_device() -> char {
    'p'
}

impl Config {
    pub fn new() -> Self {
        let conf_path = dirs::config_dir()
            .unwrap()
            .join("bluetui")
            .join("config.toml");

        let config = std::fs::read_to_string(conf_path).unwrap_or_default();
        let app_config: Config = toml::from_str(&config).unwrap();

        app_config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

use core::fmt;
use std::{path::PathBuf, process::exit};

use ratatui::layout::Flex;
use toml;

use dirs;
use serde::{
    Deserialize, Deserializer,
    de::{self, Unexpected, Visitor},
};

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_layout", deserialize_with = "deserialize_layout")]
    pub layout: Flex,

    #[serde(default = "Width::default")]
    pub width: Width,

    #[serde(default = "default_toggle_scanning")]
    pub toggle_scanning: char,

    #[serde(default)]
    pub adapter: Adapter,

    #[serde(default)]
    pub paired_device: PairedDevice,
}

#[derive(Debug, Default)]
pub enum Width {
    #[default]
    Auto,
    Size(u16),
}

struct WidthVisitor;

impl<'de> Visitor<'de> for WidthVisitor {
    type Value = Width;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("the string \"auto\" or a positive integer (u16)")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "auto" => Ok(Width::Auto),
            _ => value
                .parse::<u16>()
                .map(Width::Size)
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(value), &self)),
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match u16::try_from(value) {
            Ok(v) => Ok(Width::Size(v)),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match u16::try_from(value) {
            Ok(v) => Ok(Width::Size(v)),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Signed(value), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for Width {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(WidthVisitor)
    }
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

    #[serde(default = "default_toggle_device_trust")]
    pub toggle_trust: char,

    #[serde(default = "default_set_new_name")]
    pub rename: char,
}

impl Default for PairedDevice {
    fn default() -> Self {
        Self {
            unpair: 'u',
            toggle_trust: 't',
            rename: 'e',
        }
    }
}

fn deserialize_layout<'de, D>(deserializer: D) -> Result<Flex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    match s.as_str() {
        "Legacy" => Ok(Flex::Legacy),
        "Start" => Ok(Flex::Start),
        "End" => Ok(Flex::End),
        "Center" => Ok(Flex::Center),
        "SpaceAround" => Ok(Flex::SpaceAround),
        "SpaceBetween" => Ok(Flex::SpaceBetween),
        _ => {
            eprintln!("Wrong config: unknown layout variant {}", s);
            eprintln!(
                "The possible values are: Legacy, Start, End, Center, SpaceAround, SpaceBetween"
            );
            std::process::exit(1);
        }
    }
}

fn default_layout() -> Flex {
    Flex::SpaceAround
}

fn default_set_new_name() -> char {
    'e'
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

fn default_toggle_device_trust() -> char {
    't'
}

impl Config {
    pub fn new(config_file_path: Option<PathBuf>) -> Self {
        let conf_path = config_file_path.unwrap_or(
            dirs::config_dir()
                .unwrap()
                .join("bluetui")
                .join("config.toml"),
        );

        let config = std::fs::read_to_string(conf_path).unwrap_or_default();
        let app_config: Config = match toml::from_str(&config) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        };

        app_config
    }
}

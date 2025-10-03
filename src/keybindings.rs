use crate::config::Config;

#[derive(Debug)]
pub struct Keybinding {
    pub key: String,
    pub description: &'static str,
    pub is_section: bool,
}

pub fn build_keybindings(config: &Config) -> Vec<Keybinding> {
    let human_readable_key = |c: char| match c {
        ' ' => "Space".to_string(),
        '\n' => "Enter".to_string(),
        '\t' => "Tab".to_string(),
        other => other.to_string(),
    };

    let keys = vec![
        Keybinding {
            key: "## Global".into(),
            description: "",
            is_section: true,
        },
        Keybinding {
            key: "Esc".into(),
            description: "Dismiss different pop-ups",
            is_section: false,
        },
        Keybinding {
            key: "Tab or h/l".into(),
            description: "Switch between different sections",
            is_section: false,
        },
        Keybinding {
            key: "j or Down".into(),
            description: "Scroll down",
            is_section: false,
        },
        Keybinding {
            key: "k or Up".into(),
            description: "Scroll up",
            is_section: false,
        },
        Keybinding {
            key: human_readable_key(config.toggle_scanning),
            description: "Start/Stop scanning",
            is_section: false,
        },
        Keybinding {
            key: "?".into(),
            description: "Show help",
            is_section: false,
        },
        Keybinding {
            key: "ctrl+c or q".into(),
            description: "Quit",
            is_section: false,
        },
        Keybinding {
            key: "".into(),
            description: "",
            is_section: false,
        },
        Keybinding {
            key: "## Adapters".into(),
            description: "",
            is_section: true,
        },
        Keybinding {
            key: human_readable_key(config.adapter.toggle_pairing),
            description: "Enable/Disable the pairing",
            is_section: false,
        },
        Keybinding {
            key: human_readable_key(config.adapter.toggle_power),
            description: "Power on/off the adapter",
            is_section: false,
        },
        Keybinding {
            key: human_readable_key(config.adapter.toggle_discovery),
            description: "Enable/Disable the discovery",
            is_section: false,
        },
        Keybinding {
            key: "".into(),
            description: "",
            is_section: false,
        },
        Keybinding {
            key: "## Paired devices".into(),
            description: "",
            is_section: true,
        },
        Keybinding {
            key: human_readable_key(config.paired_device.unpair),
            description: "Unpair the device",
            is_section: false,
        },
        Keybinding {
            key: human_readable_key(config.paired_device.toggle_connect),
            description: "Connect/Disconnect the device",
            is_section: false,
        },
        Keybinding {
            key: human_readable_key(config.paired_device.toggle_trust),
            description: "Trust/Untrust the device",
            is_section: false,
        },
        Keybinding {
            key: human_readable_key(config.paired_device.rename),
            description: "Rename the device",
            is_section: false,
        },
        Keybinding {
            key: "".into(),
            description: "",
            is_section: false,
        },
        Keybinding {
            key: "## New devices".into(),
            description: "",
            is_section: true,
        },
        Keybinding {
            key: human_readable_key(config.new_device.pair),
            description: "Pair the device",
            is_section: false,
        },
    ];

    keys
}

pub fn keybindings_string(config: &Config) -> String {
    let bindings = build_keybindings(config);
    let mut s = String::from("Hotkeys:\n\n");

    for kb in bindings {
        if kb.is_section {
            s.push_str(&format!("{}\n", kb.key));
        } else if kb.key.is_empty() && kb.description.is_empty() {
            s.push('\n');
        } else {
            s.push_str(&format!("  {:18} {}\n", kb.key, kb.description));
        }
    }
    s
}

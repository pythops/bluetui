use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{app::FocusedBlock, config::Config};

pub struct Help;

impl Help {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        focused_block: FocusedBlock,
        rendering_block: Rect,
        config: Arc<Config>,
    ) {
        let nav_up = config.navigation.up.to_string();
        let nav_down = config.navigation.down.to_string();
        let toggle_scanning = config.toggle_scanning.to_string();

        let help = match focused_block {
            FocusedBlock::PairedDevices => {
                if area.width > 120 {
                    vec![Line::from(vec![
                        Span::from(nav_up.clone()).bold(),
                        Span::from(",").bold(),
                        Span::from("  Up"),
                        Span::from(" | "),
                        Span::from(nav_down.clone()).bold(),
                        Span::from(",").bold(),
                        Span::from("  Down"),
                        Span::from(" | "),
                        Span::from(toggle_scanning.clone()).bold(),
                        Span::from("  Scan on/off"),
                        Span::from(" | "),
                        Span::from(config.paired_device.unpair.to_string()).bold(),
                        Span::from("  Unpair"),
                        Span::from(" | "),
                        Span::from("󱁐  or ↵ ").bold(),
                        Span::from(" Dis/Connect"),
                        Span::from(" | "),
                        Span::from(config.paired_device.toggle_trust.to_string()).bold(),
                        Span::from(" Un/Trust"),
                        Span::from(" | "),
                        Span::from(config.paired_device.toggle_favorite.to_string()).bold(),
                        Span::from(" Un/Favorite"),
                        Span::from(" | "),
                        Span::from(config.paired_device.rename.to_string()).bold(),
                        Span::from(" Rename"),
                        Span::from(" | "),
                        Span::from("⇄").bold(),
                        Span::from(" Nav"),
                    ])]
                } else {
                    vec![
                        Line::from(vec![
                            Span::from("󱁐  or ↵ ").bold(),
                            Span::from(" Dis/Connect"),
                            Span::from(" | "),
                            Span::from(toggle_scanning.clone()).bold(),
                            Span::from("  Scan on/off"),
                            Span::from(" | "),
                            Span::from(config.paired_device.unpair.to_string()).bold(),
                            Span::from("  Unpair"),
                            Span::from(" | "),
                            Span::from(config.paired_device.toggle_favorite.to_string()).bold(),
                            Span::from(" Un/Favorite"),
                        ]),
                        Line::from(vec![
                            Span::from(config.paired_device.toggle_trust.to_string()).bold(),
                            Span::from(" Un/Trust"),
                            Span::from(" | "),
                            Span::from(config.paired_device.rename.to_string()).bold(),
                            Span::from(" Rename"),
                            Span::from(" | "),
                            Span::from(nav_up.clone()).bold(),
                            Span::from(",").bold(),
                            Span::from("  Up"),
                            Span::from(" | "),
                            Span::from(nav_down.clone()).bold(),
                            Span::from(",").bold(),
                            Span::from("  Down"),
                            Span::from(" | "),
                            Span::from("⇄").bold(),
                            Span::from(" Nav"),
                        ]),
                    ]
                }
            }
            FocusedBlock::NewDevices => vec![Line::from(vec![
                Span::from(nav_up.clone()).bold(),
                Span::from(",").bold(),
                Span::from("  Up"),
                Span::from(" | "),
                Span::from(nav_down.clone()).bold(),
                Span::from(",").bold(),
                Span::from("  Down"),
                Span::from(" | "),
                Span::from("󱁐  or ↵ ").bold(),
                Span::from(" Pair"),
                Span::from(" | "),
                Span::from(toggle_scanning.clone()).bold(),
                Span::from("  Scan on/off"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ])],
            FocusedBlock::Adapter => {
                if area.width > 80 {
                    vec![Line::from(vec![
                        Span::from(toggle_scanning.clone()).bold(),
                        Span::from("  Scan on/off"),
                        Span::from(" | "),
                        Span::from(config.adapter.toggle_pairing.to_string()).bold(),
                        Span::from(" Pairing on/off"),
                        Span::from(" | "),
                        Span::from(config.adapter.toggle_power.to_string()).bold(),
                        Span::from(" Power on/off"),
                        Span::from(" | "),
                        Span::from(config.adapter.toggle_discovery.to_string()).bold(),
                        Span::from(" Discovery on/off"),
                        Span::from(" | "),
                        Span::from("⇄").bold(),
                        Span::from(" Nav"),
                    ])]
                } else {
                    vec![
                        Line::from(vec![
                            Span::from(toggle_scanning.clone()).bold(),
                            Span::from("  Scan on/off"),
                            Span::from(" | "),
                            Span::from(config.adapter.toggle_pairing.to_string()).bold(),
                            Span::from(" Pairing on/off"),
                        ]),
                        Line::from(vec![
                            Span::from(config.adapter.toggle_power.to_string()).bold(),
                            Span::from(" Power on/off"),
                            Span::from(" | "),
                            Span::from(config.adapter.toggle_discovery.to_string()).bold(),
                            Span::from(" Discovery on/off"),
                            Span::from(" | "),
                            Span::from("⇄").bold(),
                            Span::from(" Nav"),
                        ]),
                    ]
                }
            }
            FocusedBlock::SetDeviceAliasBox => {
                vec![Line::from(vec![
                    Span::from("󱊷 ").bold(),
                    Span::from(" Discard"),
                    Span::from(" | "),
                    Span::from("↵ ").bold(),
                    Span::from(" Apply"),
                ])]
            }
            FocusedBlock::RequestConfirmation => {
                vec![Line::from(vec![
                    Span::from("↵ ").bold(),
                    Span::from(" Ok"),
                    Span::from(" | "),
                    Span::from("󱊷 ").bold(),
                    Span::from(" Discard"),
                    Span::from(" | "),
                    Span::from("⇄").bold(),
                    Span::from(" Nav"),
                ])]
            }
            FocusedBlock::EnterPinCode | FocusedBlock::EnterPasskey => {
                vec![Line::from(vec![
                    Span::from("󱊷 ").bold(),
                    Span::from(" Discard"),
                    Span::from(" | "),
                    Span::from("⇄").bold(),
                    Span::from(" Nav"),
                    Span::from(" | "),
                    Span::from("↵ ").bold(),
                    Span::from(" Submit"),
                ])]
            }
            FocusedBlock::DisplayPinCode => {
                vec![Line::from(vec![
                    Span::from(" 󱊷  or ↵ ").bold(),
                    Span::from(" Ok"),
                ])]
            }
            FocusedBlock::DisplayPasskey => {
                vec![Line::from(vec![
                    Span::from(" 󱊷  ").bold(),
                    Span::from(" Discard"),
                ])]
            }
        };
        let help = Paragraph::new(help).centered().blue();
        frame.render_widget(help, rendering_block);
    }
}
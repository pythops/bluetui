use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::{FocusedBlock, HelpSection},
    config::Config,
};

struct HelpItem<'a> {
    spans: Vec<Span<'a>>,
    x_start: u16,
    x_end: u16,
}

impl<'a> HelpItem<'a> {
    fn new(spans: Vec<Span<'a>>) -> Self {
        let width: u16 = spans.iter().map(|s| s.content.len() as u16).sum();
        Self {
            spans,
            x_start: 0,
            x_end: width,
        }
    }

    fn width(&self) -> u16 {
        self.x_end - self.x_start
    }

    fn set_position(&mut self, x_start: u16) {
        let width = self.width();
        self.x_start = x_start;
        self.x_end = x_start + width;
    }

    fn get_spans(&self) -> Vec<Span<'a>> {
        self.spans.clone()
    }

    fn label(&self) -> String {
        self.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn to_section(&self, y: u16) -> HelpSection {
        HelpSection {
            label: self.label().trim().to_string(),
            x_start: self.x_start,
            x_end: self.x_end,
            y,
        }
    }
}

pub struct Help;

impl Help {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        focused_block: FocusedBlock,
        rendering_block: Rect,
        config: Arc<Config>,
    ) -> Vec<HelpSection> {
        let mut section_indexes: Vec<(HelpItem, u16)> = Vec::new(); // (item, line_index)

        let help = match focused_block {
            FocusedBlock::PairedDevices => {
                if area.width > 120 {
                    let mut up_item =
                        HelpItem::new(vec![Span::from("k,").bold(), Span::from("  Up")]);
                    let mut down_item =
                        HelpItem::new(vec![Span::from("j,").bold(), Span::from("  Down")]);
                    let mut scan_item =
                        HelpItem::new(vec![Span::from("s").bold(), Span::from("  Scan on/off")]);
                    let mut unpair_item = HelpItem::new(vec![
                        Span::from(config.paired_device.unpair.to_string()).bold(),
                        Span::from("  Unpair----"),
                    ]);
                    let mut connect_item =
                        HelpItem::new(vec![Span::from("󱁐  or ↵ ").bold(), Span::from(" Dis/Connect")]);
                    let mut trust_item = HelpItem::new(vec![
                        Span::from(config.paired_device.toggle_trust.to_string()).bold(),
                        Span::from(" Un/Trust"),
                    ]);
                    let mut favorite_item = HelpItem::new(vec![
                        Span::from(config.paired_device.toggle_favorite.to_string()).bold(),
                        Span::from(" Un/Favorite"),
                    ]);
                    let mut rename_item = HelpItem::new(vec![
                        Span::from(config.paired_device.rename.to_string()).bold(),
                        Span::from(" Rename"),
                    ]);
                    let mut nav_item =
                        HelpItem::new(vec![Span::from("⇄").bold(), Span::from(" Nav")]);

                    let separator = Span::from(" | ");

                    let mut all_spans: Vec<Span> = Vec::new();
                    all_spans.extend(up_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(down_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(scan_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(unpair_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(connect_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(trust_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(favorite_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(rename_item.get_spans());
                    all_spans.push(separator.clone());
                    all_spans.extend(nav_item.get_spans());

                    // Calculate start_x for centered line
                    let total_width: u16 = all_spans.iter().map(|s| s.content.len() as u16).sum();
                    let start_x =
                        rendering_block.x + (rendering_block.width.saturating_sub(total_width)) / 2;

                    // Set positions on each item
                    let sep_width = separator.content.len() as u16;
                    let mut current_x = start_x;

                    up_item.set_position(current_x);
                    current_x += up_item.width() + sep_width;
                    down_item.set_position(current_x);
                    current_x += down_item.width() + sep_width;
                    scan_item.set_position(current_x);
                    current_x += scan_item.width() + sep_width;
                    unpair_item.set_position(current_x);
                    current_x += unpair_item.width() + sep_width;
                    connect_item.set_position(current_x);
                    current_x += connect_item.width() + sep_width;
                    trust_item.set_position(current_x);
                    current_x += trust_item.width() + sep_width;
                    favorite_item.set_position(current_x);
                    current_x += favorite_item.width() + sep_width;
                    rename_item.set_position(current_x);
                    current_x += rename_item.width() + sep_width;
                    nav_item.set_position(current_x);

                    // Add all items to helpItem_lines with line index 0 since it's a single line
                    section_indexes.push((up_item, 0));
                    section_indexes.push((down_item, 0));
                    section_indexes.push((scan_item, 0));
                    section_indexes.push((unpair_item, 0));
                    section_indexes.push((connect_item, 0));
                    section_indexes.push((trust_item, 0));
                    section_indexes.push((favorite_item, 0));
                    section_indexes.push((rename_item, 0));
                    section_indexes.push((nav_item, 0));

                    vec![Line::from(all_spans)]
                } else {
                    vec![
                        Line::from(vec![
                            Span::from("󱁐  or ↵ ").bold(),
                            Span::from(" Dis/Connect"),
                            Span::from(" | "),
                            Span::from("s").bold(),
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
                            Span::from("k,").bold(),
                            Span::from("  Up"),
                            Span::from(" | "),
                            Span::from("j,").bold(),
                            Span::from("  Down"),
                            Span::from(" | "),
                            Span::from("⇄").bold(),
                            Span::from(" Nav"),
                        ]),
                    ]
                }
            }
            FocusedBlock::NewDevices => vec![Line::from(vec![
                Span::from("k,").bold(),
                Span::from("  Up"),
                Span::from(" | "),
                Span::from("j,").bold(),
                Span::from("  Down"),
                Span::from(" | "),
                Span::from("󱁐  or ↵ ").bold(),
                Span::from(" Pair"),
                Span::from(" | "),
                Span::from("s").bold(),
                Span::from("  Scan on/off"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ])],
            FocusedBlock::Adapter => {
                if area.width > 80 {
                    vec![Line::from(vec![
                        Span::from("s").bold(),
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
                            Span::from("s").bold(),
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

        let mut sections = Vec::new();
        for (item, line_idx) in section_indexes {
            let y = rendering_block.y + line_idx;
            sections.push(item.to_section(y));
        }

        let help = Paragraph::new(help).centered().blue();
        frame.render_widget(help, rendering_block);

        sections
    }
}

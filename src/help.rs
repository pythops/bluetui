use std::sync::Arc;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};

use crate::{app::ColorMode, config::Config};

#[derive(Debug)]
pub struct Help {
    block_height: usize,
    state: TableState,
    keys: Vec<(Cell<'static>, &'static str)>,
}

impl Help {
    pub fn new(config: Arc<Config>) -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        Self {
            block_height: 0,
            state,
            keys: vec![
                (
                    Cell::from("## Global").style(Style::new().bold().fg(Color::Yellow)),
                    "",
                ),
                (Cell::from("Esc").bold(), "Dismiss different pop-ups"),
                (
                    Cell::from("Tab").bold(),
                    "Switch between different sections",
                ),
                (Cell::from("j or Down").bold(), "Scroll down"),
                (Cell::from("k or Up").bold(), "Scroll up"),
                (
                    Cell::from(config.toggle_scanning.to_string()).bold(),
                    "Start/Stop scanning",
                ),
                (Cell::from("?").bold(), "Show help"),
                (Cell::from("ctrl+c or q").bold(), "Quit"),
                (Cell::from(""), ""),
                (
                    Cell::from("## Adapters").style(Style::new().bold().fg(Color::Yellow)),
                    "",
                ),
                (
                    Cell::from(config.adapter.toggle_pairing.to_string()).bold(),
                    "Enable/Disable the pairing",
                ),
                (
                    Cell::from(config.adapter.toggle_power.to_string()).bold(),
                    "Power on/off the adapter",
                ),
                (
                    Cell::from(config.adapter.toggle_discovery.to_string()).bold(),
                    "Enable/Disable the discovery",
                ),
                (Cell::from(""), ""),
                (
                    Cell::from("## Paired devices").style(Style::new().bold().fg(Color::Yellow)),
                    "",
                ),
                (
                    Cell::from(config.paired_device.unpair.to_string()).bold(),
                    "Unpair the device",
                ),
                (
                    Cell::from({
                        if config.paired_device.toggle_connect == ' ' {
                            "Space".to_string()
                        } else {
                            config.paired_device.toggle_connect.to_string()
                        }
                    })
                    .bold(),
                    "Connect/Disconnect the device",
                ),
                (
                    Cell::from(config.paired_device.toggle_trust.to_string()).bold(),
                    "Trust/Untrust the device",
                ),
                (
                    Cell::from(config.paired_device.rename.to_string()).bold(),
                    "Rename the device",
                ),
                (Cell::from(""), ""),
                (
                    Cell::from("## New devices").style(Style::default().bold().fg(Color::Yellow)),
                    "",
                ),
                (
                    Cell::from(config.new_device.pair.to_string()).bold(),
                    "Pair the device",
                ),
            ],
        }
    }

    pub fn scroll_down(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.keys.len().saturating_sub(self.block_height - 6) {
                    i
                } else {
                    i + 1
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }
    pub fn scroll_up(&mut self) {
        let i = match self.state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }

    pub fn render(&mut self, frame: &mut Frame, color_mode: ColorMode) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(28),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(frame.area());

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(70),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[1])[1];

        self.block_height = block.height as usize;

        let widths = [Constraint::Length(20), Constraint::Max(40)];
        let rows: Vec<Row> = self
            .keys
            .iter()
            .map(|key| {
                Row::new(vec![key.0.to_owned(), key.1.into()]).style(match color_mode {
                    ColorMode::Dark => Style::default().fg(Color::White),
                    ColorMode::Light => Style::default().fg(Color::Black),
                })
            })
            .collect();
        let rows_len = self.keys.len().saturating_sub(self.block_height - 6);

        let table = Table::new(rows, widths).block(
            Block::default()
                .padding(Padding::uniform(2))
                .title(" Help ")
                .title_style(Style::default().bold().fg(Color::Green))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
        );

        frame.render_widget(Clear, block);
        frame.render_stateful_widget(table, block, &mut self.state);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(rows_len).position(self.state.selected().unwrap_or_default());
        frame.render_stateful_widget(
            scrollbar,
            block.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

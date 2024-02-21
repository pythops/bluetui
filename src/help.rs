use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Padding, Row, Table, TableState},
    Frame,
};

#[derive(Debug)]
pub struct Help {
    block_height: usize,
    state: TableState,
    keys: &'static [(&'static str, &'static str)],
}

impl Default for Help {
    fn default() -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        Self {
            block_height: 0,
            state,
            keys: &[
                ("# Global", ""),
                ("Esc", "Dismiss help pop-up"),
                ("Tab", "Switch between different sections"),
                ("j or Down", "Scroll down"),
                ("k or Up", "Scroll up"),
                ("s", "Start/Stop scanning"),
                ("?", "Show help"),
                ("", ""),
                ("# Adapters", ""),
                ("p", "Enable/Disable the pairing"),
                ("o", "Power on/off the adapter"),
                ("d", "Enable/Disable the discovery"),
                ("", ""),
                ("# Paired devices", ""),
                ("u", "Unpair the device"),
                ("Space", "Connect/Disconnect the device"),
                ("t", "Trust/Untrust the device"),
                ("", ""),
                ("# New devices", ""),
                ("p", "Pair the device"),
            ],
        }
    }
}

impl Help {
    pub fn new() -> Self {
        Self::default()
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
            Some(i) => {
                if i > 1 {
                    i - 1
                } else {
                    0
                }
            }
            None => 1,
        };
        *self.state.offset_mut() = i;
        self.state.select(Some(i));
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let block = help_rect(frame.size());

        self.block_height = block.height as usize;
        let widths = [Constraint::Length(15), Constraint::Min(60)];
        let rows: Vec<Row> = self
            .keys
            .iter()
            .map(|key| Row::new(vec![key.0, key.1]))
            .collect();

        let table = Table::new(rows, widths).block(
            Block::default()
                .padding(Padding::uniform(2))
                .title(" Help ")
                .title_style(Style::default().bold())
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_style(Style::default()),
        );

        frame.render_widget(Clear, block);
        frame.render_stateful_widget(table, block, &mut self.state);
    }
}

pub fn help_rect(r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(35),
                Constraint::Min(10),
                Constraint::Percentage(35),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - 80) / 2),
                Constraint::Min(80),
                Constraint::Length((r.width - 80) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

use std::sync::Arc;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{app::ColorMode, config::Config};

#[derive(Debug)]
pub struct AliasFilter{ 
    pub filter: Option<String>,
    state: TableState,
}

impl AliasFilter {
    pub fn new(config: Arc<Config>) -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        Self {
            state,
            filter: None
        }
    }

    pub fn render(&mut self, frame: &mut Frame, color_mode: ColorMode) {
        let widths = [
            Constraint::Length(20),
            Constraint::Length(20),
        ];

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

        let row = Row::new(vec!["asdf"]);
        let table = Table::new([row], widths);

        frame.render_widget(table, block);
    }
}

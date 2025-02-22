use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin}, style::{Color, Style, Stylize}, text::ToText, widgets::{Block, BorderType, Borders, Padding, Row, Table, TableState}, Frame
};
use tui_input::{backend::crossterm::EventHandler, Input, InputRequest};

use crate::{app::ColorMode, config::Config};


#[derive(Debug)]
pub struct AliasFilter{ 
    pub filter: Option<String>,
    input: Input,
    state: TableState,
}

impl AliasFilter {
    pub fn new(config: Arc<Config>) -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        let input = Input::new("".to_string());

        Self {
            state,
            input,
            filter: None
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let req = InputRequest::InsertChar(c);
        self.input.handle(req);
        self.filter = Some(self.input.value().to_string());
    }

    pub fn delete_char(&mut self) {
        let req = InputRequest::DeletePrevChar;
        self.input.handle(req);
        self.filter = Some(self.input.value().to_string());
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
                Constraint::Length(60),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[1])[1];

        let text = match &self.filter {
            Some(f) => f.to_string(),
            None => "".to_string()
        };

        let row = Row::new(vec![text]);

        let table = Table::new([row], [Constraint::Length(20)]).block(
            Block::default()
                .padding(Padding::uniform(2))
                .title(" Filter Device Names ")
                .title_style(Style::default().bold().fg(Color::Green))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
        );

        frame.render_stateful_widget(table, block, &mut self.state);
    }
}

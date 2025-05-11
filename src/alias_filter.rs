use std::sync::Arc;
use std::time::Instant;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Clear, Padding, Row, Table, TableState},
    Frame,
};
use tui_input::{Input, InputRequest};

use crate::config::Config;

#[derive(Debug)]
pub struct AliasFilter {
    pub filter: Option<String>,
    input: Input,
    state: TableState,
    start: Instant,
}

impl AliasFilter {
    pub fn new(_config: Arc<Config>) -> Self {
        let mut state = TableState::new().with_offset(0);
        state.select(Some(0));

        let input = Input::new("".to_string());

        let start = Instant::now();

        Self {
            state,
            input,
            filter: None,
            start,
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

    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Length(5),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(frame.area());

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Min(100),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[2])[1];

        let mut text = match &self.filter {
            Some(f) => f.to_string(),
            None => "".to_string(),
        };

        self.insert_cursor(&mut text);

        let row = Row::new(vec![text]);

        let table = Table::new([row], [Constraint::Length(20)]).block(
            Block::default()
                .padding(Padding::uniform(1))
                .title(" Filter Device Names ")
                .title_style(Style::default().bold().fg(Color::Green))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .style(Style::default())
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
        );

        frame.render_widget(Clear, block);
        frame.render_stateful_widget(table, block, &mut self.state);
    }

    fn insert_cursor(&self, s: &mut String) {
        let time = Instant::now().duration_since(self.start).as_secs();

        let c = if time % 2 == 0 { '_' } else { ' ' };

        s.insert(self.input.cursor(), c);
    }
}

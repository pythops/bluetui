use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use bluer::Address;

use crate::{agent::AuthAgent, app::AppResult, config::Config};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DisplayPinCode {
    pub adapter: String,
    pub device: Address,
    pub pin_code: String,
}

impl DisplayPinCode {
    pub fn new(adapter: String, device: Address, pin_code: String) -> Self {
        Self {
            adapter,
            device,
            pin_code,
        }
    }

    pub async fn submit(&mut self, agent: &AuthAgent) -> AppResult<()> {
        agent.tx_display_pin_code.send(()).await?;
        agent
            .event_sender
            .send(crate::event::Event::DisplayPinCodeSeen)?;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, config: Arc<Config>) {
        let block = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .margin(2)
            .split(area)[1];

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(60),
                Constraint::Fill(1),
            ])
            .margin(1)
            .split(block)[1];

        let message = vec![
            Line::from(format!("Pin Code for the device {} ", self.device)).centered(),
            Line::from(""),
            Line::from(self.pin_code.clone())
                .centered()
                .bold()
                .bg(config.colors.highlight_bg),
        ];

        let message = Paragraph::new(message).centered();

        frame.render_widget(Clear, block);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(config.colors.focused_border)),
            block,
        );
        frame.render_widget(
            message,
            block.inner(Margin {
                horizontal: 0,
                vertical: 2,
            }),
        );
    }
}

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::Modifier,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use bluer::Address;

use crate::{agent::AuthAgent, app::AppResult, theme::Theme};

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

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .margin(2)
        .split(area)[1];

        let block = Layout::horizontal([
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
                .style(theme.input.add_modifier(Modifier::BOLD)),
        ];

        let message = Paragraph::new(message).centered().style(theme.popup_text);

        frame.render_widget(Clear, block);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(theme.popup_border),
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

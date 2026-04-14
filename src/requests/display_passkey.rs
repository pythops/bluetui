use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use bluer::Address;

use crate::{agent::AuthAgent, app::AppResult, theme::Theme};

#[derive(Debug, Clone)]
pub struct DisplayPasskey {
    pub adapter: String,
    pub device: Address,
    pub passkey: u32,
    pub entered: u16,
}

impl DisplayPasskey {
    pub fn new(adapter: String, device: Address, passkey: u32, entered: u16) -> Self {
        Self {
            adapter,
            device,
            passkey,
            entered,
        }
    }

    pub async fn cancel(&mut self, agent: &AuthAgent) -> AppResult<()> {
        agent.tx_cancel.send(()).await?;
        agent
            .event_sender
            .send(crate::event::Event::DisplayPasskeyCanceled)?;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(12),
            Constraint::Fill(1),
        ])
        .margin(2)
        .split(area)[1];

        let block = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Max(70),
            Constraint::Fill(1),
        ])
        .margin(1)
        .split(block)[1];

        let message = vec![
            Line::from(format!("Authentication for the device {}", self.device)).centered(),
            Line::from(""),
            Line::from(vec![Span::from(
                "Enter the following passkey on the remote device",
            )])
            .centered(),
            Line::from(""),
            Line::from(self.passkey.to_string()).style(theme.input.add_modifier(Modifier::BOLD)),
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

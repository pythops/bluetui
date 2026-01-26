use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear},
};

use bluer::Address;

use crate::{agent::AuthAgent, app::AppResult, config::Config};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Confirmation {
    pub adapter: String,
    pub device: Address,
    pub passkey: u32,
    confirmed: bool,
}

impl Confirmation {
    pub fn new(adapter: String, device: Address, passkey: u32) -> Self {
        Self {
            adapter,
            device,
            passkey,
            confirmed: true,
        }
    }

    pub async fn submit(&mut self, agent: &AuthAgent) -> AppResult<()> {
        agent.tx_request_confirmation.send(self.confirmed).await?;
        agent
            .event_sender
            .send(crate::event::Event::ConfirmationSubmitted)?;
        Ok(())
    }

    pub async fn cancel(&mut self, agent: &AuthAgent) -> AppResult<()> {
        agent.tx_cancel.send(()).await?;
        agent
            .event_sender
            .send(crate::event::Event::ConfirmationSubmitted)?;
        Ok(())
    }

    pub fn toggle_select(&mut self) {
        self.confirmed = !self.confirmed;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, config: Arc<Config>) {
        let block = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Fill(1),
            ])
            .split(area);

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(70),
                Constraint::Fill(1),
            ])
            .split(block[1])[1];

        let (message_block, choices_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Length(2),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(block);

            (chunks[1], chunks[3])
        };

        let message = Text::from(vec![
            Line::from(vec![
                Span::from("Authentication required for the device "),
                Span::from(self.device.to_string()),
            ])
            .centered(),
            Line::from(""),
            Line::from(vec![
                Span::from("Confirm Passkey "),
                Span::styled(
                    format!("{:06}", self.passkey),
                    Style::new().bg(config.colors.highlight_bg).bold(),
                ),
            ])
            .centered(),
        ]);

        let choice = {
            if self.confirmed {
                Line::from(vec![
                    Span::from("No").style(Style::default()),
                    Span::from("        "),
                    Span::from("Yes")
                        .style(Style::default().bg(config.colors.info))
                        .bold(),
                ])
            } else {
                Line::from(vec![
                    Span::from("No")
                        .style(Style::default().bg(config.colors.info))
                        .bold(),
                    Span::from("        "),
                    Span::from("Yes").style(Style::default()),
                ])
            }
        };

        frame.render_widget(Clear, block);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(config.colors.focused_border)),
            block,
        );
        frame.render_widget(message, message_block);
        frame.render_widget(choice.centered(), choices_block);
    }
}

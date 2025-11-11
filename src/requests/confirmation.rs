use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear},
};

use bluer::Address;

use crate::{agent::AuthAgent, app::AppResult};

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

    pub fn render(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Fill(1),
            ])
            .split(frame.area());

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(80),
                Constraint::Fill(1),
            ])
            .split(layout[1])[1];

        let (message_area, choices_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(block);

            (chunks[1], chunks[3])
        };

        let message = Text::from(format!(
            "Is Passkey {:06} correct for the device {}  ?",
            self.passkey, self.device,
        ))
        .centered();

        let choice = {
            if self.confirmed {
                Line::from(vec![
                    Span::from("[No]").style(Style::default()),
                    Span::from("        "),
                    Span::from("[Yes]").style(Style::default().bg(Color::DarkGray)),
                ])
            } else {
                Line::from(vec![
                    Span::from("[No]").style(Style::default().bg(Color::DarkGray)),
                    Span::from("        "),
                    Span::from("[Yes]").style(Style::default()),
                ])
            }
        };

        frame.render_widget(Clear, block);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
            block,
        );
        frame.render_widget(message, message_area);
        frame.render_widget(choice.centered(), choices_area);
    }
}

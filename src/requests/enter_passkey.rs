use crossterm::event::{KeyCode, KeyEvent};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List},
};

use bluer::Address;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    agent::AuthAgent,
    app::AppResult,
    config::Config,
    event::Event,
    requests::{pad_str, pad_string},
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FocusedSection {
    #[default]
    Input,
    Submit,
}

#[derive(Debug, Clone)]
pub struct EnterPasskey {
    pub adapter: String,
    pub device: Address,
    focused_section: FocusedSection,
    passkey: UserInputField,
}

#[derive(Debug, Clone, Default)]
struct UserInputField {
    field: Input,
    error: Option<String>,
}

impl EnterPasskey {
    pub fn new(adapter: String, device: Address) -> Self {
        Self {
            adapter,
            device,
            focused_section: FocusedSection::default(),
            passkey: UserInputField::default(),
        }
    }

    pub async fn submit(&mut self, agent: &AuthAgent) -> AppResult<()> {
        self.validate();

        if self.passkey.error.is_some() {
            return Ok(());
        }

        agent
            .tx_passkey
            .send(self.passkey.field.value().parse::<u32>().unwrap())
            .await?;

        agent.event_sender.send(Event::PasskeySumitted)?;
        Ok(())
    }

    pub async fn cancel(&mut self, agent: &AuthAgent) -> AppResult<()> {
        agent.tx_cancel.send(()).await?;
        agent.event_sender.send(Event::PasskeySumitted)?;
        Ok(())
    }

    pub fn validate(&mut self) {
        self.passkey.error = None;
        if self.passkey.field.value().is_empty() {
            self.passkey.error = Some("Required field.".to_string());
            return;
        }

        if self.passkey.field.value().len() > 6 {
            self.passkey.error =
                Some("Passkey should be a numeric value between 0-999999".to_string());
            return;
        }

        if self.passkey.field.value().parse::<u32>().is_err() {
            self.passkey.error =
                Some("Passkey should be a numeric value between 0-999999".to_string());
        }
    }

    pub async fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        agent: &AuthAgent,
    ) -> AppResult<()> {
        match key_event.code {
            KeyCode::Tab | KeyCode::BackTab => {
                if self.focused_section == FocusedSection::Input {
                    self.focused_section = FocusedSection::Submit;
                } else {
                    self.focused_section = FocusedSection::Input;
                }
            }
            _ => match self.focused_section {
                FocusedSection::Submit => {
                    if let KeyCode::Enter = key_event.code {
                        self.submit(agent).await?;
                    }
                }

                _ => {
                    self.passkey
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
            },
        }

        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, config: Arc<Config>) {
        let layout = Layout::default()
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
            .split(layout[1])[1];

        let (message_block, input_block, submit_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1), // message
                        Constraint::Length(1),
                        Constraint::Length(3), // input
                        Constraint::Length(1),
                        Constraint::Length(1), // enter
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(block);

            (chunks[1], chunks[3], chunks[5])
        };

        let input_block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(5), Constraint::Fill(1), Constraint::Max(5)].as_ref())
            .flex(ratatui::layout::Flex::Center)
            .split(input_block)[1];

        let message = Text::from(format!(
            "Enter the Passkey for the device {} on {}",
            self.device, self.adapter,
        ))
        .centered();

        let items = vec![
            Line::from(vec![
                {
                    if self.focused_section == FocusedSection::Input {
                        Span::from("Passkey").fg(config.colors.focused_header).bold()
                    } else {
                        Span::from("Passkey")
                    }
                },
                Span::from("  "),
                Span::from(pad_string(format!(" {}", self.passkey.field.value()), 60))
                    .bg(config.colors.highlight_bg),
            ]),
            Line::from(vec![Span::from(pad_str(" ", 9)), {
                if let Some(error) = &self.passkey.error {
                    Span::from(pad_str(error, 60))
                } else {
                    Span::from("")
                }
            }])
            .fg(config.colors.error),
        ];

        let user_input = List::new(items);

        let submit = if self.focused_section == FocusedSection::Submit {
            Text::from("Submit").centered().bold().fg(config.colors.focused_header)
        } else {
            Text::from("Submit").centered()
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
        frame.render_widget(user_input, input_block);
        frame.render_widget(submit, submit_block);
    }
}

use crossterm::event::{KeyCode, KeyEvent};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List},
};

use bluer::Address;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{agent::AuthAgent, app::AppResult, event::Event, requests::pad_string};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FocusedSection {
    #[default]
    Input,
    Submit,
}

#[derive(Debug, Clone)]
pub struct EnterPinCode {
    pub adapter: String,
    pub device: Address,
    focused_section: FocusedSection,
    pin_code: UserInputField,
}

#[derive(Debug, Clone, Default)]
struct UserInputField {
    field: Input,
    error: Option<String>,
}

impl EnterPinCode {
    pub fn new(adapter: String, device: Address) -> Self {
        Self {
            adapter,
            device,
            focused_section: FocusedSection::default(),
            pin_code: UserInputField::default(),
        }
    }

    pub async fn submit(&mut self, agent: &AuthAgent) -> AppResult<()> {
        self.validate();

        if self.pin_code.error.is_some() {
            return Ok(());
        }

        agent
            .tx_pin_code
            .send(self.pin_code.field.value().to_string())
            .await?;

        agent.event_sender.send(Event::PinCodeSumitted)?;
        Ok(())
    }

    pub async fn cancel(&mut self, agent: &AuthAgent) -> AppResult<()> {
        agent.tx_cancel.send(()).await?;
        agent.event_sender.send(Event::PinCodeSumitted)?;
        Ok(())
    }

    pub fn validate(&mut self) {
        self.pin_code.error = None;
        if self.pin_code.field.value().is_empty() {
            self.pin_code.error = Some("Required field.".to_string());
            return;
        }

        if self.pin_code.field.value().len() > 16 {
            self.pin_code.error =
                Some("Pin Code should be a string of 1-16 characters length".to_string());
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
                    self.pin_code
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
            },
        }

        Ok(())
    }

    pub fn render(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Fill(1),
            ])
            .split(frame.area());

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
            "Enter the PIN Code for the device {} on {}",
            self.device, self.adapter,
        ))
        .centered();

        let items = vec![
            Line::from(vec![
                {
                    if self.focused_section == FocusedSection::Input {
                        Span::from("Pin Code").green().bold()
                    } else {
                        Span::from("Pin Code")
                    }
                },
                Span::from("  "),
                Span::from(pad_string(
                    format!(" {}", self.pin_code.field.value()).as_str(),
                    60,
                ))
                .bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(pad_string(" ", 10)), {
                if let Some(error) = &self.pin_code.error {
                    Span::from(pad_string(error, 60))
                } else {
                    Span::from("")
                }
            }])
            .red(),
        ];

        let user_input = List::new(items);

        let submit = if self.focused_section == FocusedSection::Submit {
            Text::from("Submit").centered().bold().green()
        } else {
            Text::from("Submit").centered()
        };

        frame.render_widget(Clear, block);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
            block,
        );

        frame.render_widget(message, message_block);
        frame.render_widget(user_input, input_block);
        frame.render_widget(submit, submit_block);
    }
}

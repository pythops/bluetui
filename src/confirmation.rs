use std::sync::mpsc::channel;
use std::sync::{atomic::AtomicBool, Arc};

use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear};
use ratatui::Frame;

#[derive(Debug)]
pub struct PairingConfirmation {
    pub confirmed: bool,
    pub display: Arc<AtomicBool>,
    pub message: Option<String>,
    pub user_confirmation_sender: async_channel::Sender<bool>,
    pub user_confirmation_receiver: async_channel::Receiver<bool>,
    pub confirmation_message_sender: std::sync::mpsc::Sender<String>,
    pub confirmation_message_receiver: std::sync::mpsc::Receiver<String>,
}

impl Default for PairingConfirmation {
    fn default() -> Self {
        Self::new()
    }
}

impl PairingConfirmation {
    pub fn new() -> Self {
        let (user_confirmation_sender, user_confirmation_receiver) = async_channel::unbounded();

        let (confirmation_message_sender, confirmation_message_receiver) = channel::<String>();
        Self {
            confirmed: true,
            display: Arc::new(AtomicBool::new(false)),
            message: None,
            user_confirmation_sender,
            user_confirmation_receiver,
            confirmation_message_sender,
            confirmation_message_receiver,
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if self.message.is_none() {
            let msg = self.confirmation_message_receiver.recv().unwrap();
            self.message = Some(msg);
        }

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

        let (text_area, choices_area) = {
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

        let (yes_area, no_area) = {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(30),
                        Constraint::Length(5),
                        Constraint::Min(1),
                        Constraint::Length(5),
                        Constraint::Percentage(30),
                    ]
                    .as_ref(),
                )
                .split(choices_area);

            (chunks[1], chunks[3])
        };

        let text = Text::from(self.message.clone().unwrap_or_default())
            .style(Style::default().fg(Color::White));

        let (yes, no) = {
            if self.confirmed {
                let no = Span::from("[No]").style(Style::default());
                let yes = Span::from("[Yes]").style(Style::default().bg(Color::DarkGray));
                (yes, no)
            } else {
                let no = Span::from("[No]").style(Style::default().bg(Color::DarkGray));
                let yes = Span::from("[Yes]").style(Style::default());
                (yes, no)
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
        frame.render_widget(text.alignment(Alignment::Center), text_area);
        frame.render_widget(yes, yes_area);
        frame.render_widget(no, no_area);
    }
}

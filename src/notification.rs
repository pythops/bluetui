use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{app::AppResult, event::Event};

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub ttl: u16,
}

#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Error,
    Warning,
    Info,
}

impl Notification {
    pub fn render(&self, index: usize, frame: &mut Frame) {
        let (color, title) = match self.level {
            NotificationLevel::Info => (Color::Green, "Info"),
            NotificationLevel::Warning => (Color::Yellow, "Warning"),
            NotificationLevel::Error => (Color::Red, "Error"),
        };

        let mut text = Text::from(vec![
            Line::from(title).style(Style::new().fg(color).add_modifier(Modifier::BOLD))
        ]);

        text.extend(Text::from(self.message.as_str()));

        let notification_height = text.height() as u16 + 4;

        let block = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(color)),
            );

        let area = notification_rect(index as u16, notification_height, frame.size());

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }
    pub fn send(
        message: String,
        level: NotificationLevel,
        sender: UnboundedSender<Event>,
    ) -> AppResult<()> {
        let notif = Notification {
            message,
            level,
            ttl: 3,
        };

        sender.send(Event::Notification(notif))?;

        Ok(())
    }
}

pub fn notification_rect(offset: u16, notification_height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1 + notification_height * offset),
                Constraint::Length(notification_height),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(30),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

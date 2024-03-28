use ratatui::Frame;

use crate::app::{App, FocusedBlock};

pub fn render(app: &mut App, frame: &mut Frame) {
    // App
    app.render(frame);

    // Help
    if let FocusedBlock::Help = app.focused_block {
        app.help.render(frame, app.color_mode);
    }

    // Notifications
    for (index, notification) in app.notifications.iter().enumerate() {
        notification.render(index, frame);
    }
}

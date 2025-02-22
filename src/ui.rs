use ratatui::Frame;

use crate::app::{App, FocusedBlock};

pub fn render(app: &mut App, frame: &mut Frame) {
    // App
    app.render(frame);

    match app.focused_block {
        FocusedBlock::Help => app.help.render(frame, app.color_mode),
        FocusedBlock::AliasFilterPopup => app.alias_filter.render(frame),
        FocusedBlock::SetDeviceAliasBox => app.render_set_alias(frame),
        _ => {}
    }

    // Notifications
    for (index, notification) in app.notifications.iter().enumerate() {
        notification.render(index, frame);
    }
}

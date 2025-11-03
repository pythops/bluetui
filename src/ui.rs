use ratatui::Frame;

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    app.render(frame);

    for (index, notification) in app.notifications.iter().enumerate() {
        notification.render(index, frame, app.area(frame));
    }
}

use crate::bluetooth::Controller;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Clear, Padding, Paragraph, TableState},
};
use tui_input::Input;

pub fn render_set_alias(
    controllers: &[Controller],
    controller_state: &TableState,
    paired_devices_state: &TableState,
    new_alias: &Input,
    frame: &mut Frame,
    area: Rect,
) {
    let center_cutout = area.centered(Constraint::Max(70), Constraint::Length(6));

    let [_, message_area, full_input_area, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .areas(center_cutout);

    let input_area = full_input_area.centered_horizontally(Constraint::Percentage(60));

    frame.render_widget(Clear, center_cutout);
    frame.render_widget(
        Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(Style::default().green()),
        center_cutout,
    );

    if let Some(selected_controller) = controller_state.selected() {
        let controller = &controllers[selected_controller];
        if let Some(index) = paired_devices_state.selected() {
            let name = controller.paired_devices[index].alias.as_str();

            let message_line =
                Line::from(vec!["Enter the new name for ".into(), name.bold().italic()]);

            let message_paragraph = Paragraph::new(message_line)
                .centered()
                .block(Block::new().padding(Padding::horizontal(2)));

            let alias_input = Paragraph::new(new_alias.value())
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::White))
                .block(Block::new().on_dark_gray().padding(Padding::horizontal(2)));

            frame.render_widget(message_paragraph, message_area);
            frame.render_widget(alias_input, input_area);
        }
    }
}

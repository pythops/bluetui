use crate::bluetooth::Controller;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, TableState},
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
    let block = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(6),
        Constraint::Fill(1),
    ])
    .split(area);

    let block = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Max(70),
        Constraint::Fill(1),
    ])
    .split(block[1])[1];

    let (text_block, alias_block) = {
        let chunks = Layout::vertical(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(block);

        let area1 = Layout::horizontal(
            [
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

        let area2 = Layout::horizontal(
            [
                Constraint::Percentage(20),
                Constraint::Fill(1),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(chunks[2]);

        (area1[1], area2[1])
    };

    frame.render_widget(Clear, block);
    frame.render_widget(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().green())
            .border_style(Style::default().fg(Color::Green)),
        block,
    );

    if let Some(selected_controller) = controller_state.selected() {
        let controller = &controllers[selected_controller];
        if let Some(index) = paired_devices_state.selected() {
            let name = controller.paired_devices[index].alias.as_str();

            let text = Line::from(vec![
                Span::from("Enter the new name for "),
                Span::styled(
                    name,
                    Style::default().add_modifier(Modifier::BOLD | Modifier::ITALIC),
                ),
            ]);

            let msg = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::White))
                .block(Block::new().padding(Padding::horizontal(2)));

            let alias = Paragraph::new(new_alias.value())
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::White))
                .block(
                    Block::new()
                        .bg(Color::DarkGray)
                        .padding(Padding::horizontal(2)),
                );

            frame.render_widget(msg, text_block);
            frame.render_widget(alias, alias_block);
        }
    }
}

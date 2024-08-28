use bluer::{
    agent::{Agent, AgentHandle},
    Session,
};
use futures::FutureExt;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
    Frame,
};
use std::sync::mpsc::channel;
use tui_input::Input;

use crate::{
    bluetooth::{request_confirmation, Controller},
    config::Config,
    help::Help,
    notification::Notification,
    spinner::Spinner,
};
use std::{
    error,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedBlock {
    Adapter,
    PairedDevices,
    NewDevices,
    Help,
    PassKeyConfirmation,
    SetDeviceAliasBox,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Dark,
    Light,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub session: Arc<Session>,
    pub agent: AgentHandle,
    pub help: Help,
    pub spinner: Spinner,
    pub notifications: Vec<Notification>,
    pub controllers: Vec<Controller>,
    pub controller_state: TableState,
    pub paired_devices_state: TableState,
    pub new_devices_state: TableState,
    pub focused_block: FocusedBlock,
    pub pairing_confirmation: PairingConfirmation,
    pub color_mode: ColorMode,
    pub new_alias: Input,
}

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
}

pub fn popup(r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Length(5),
                Constraint::Percentage(45),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - 80) / 2),
                Constraint::Min(80),
                Constraint::Length((r.width - 80) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
impl App {
    pub fn reset_devices_state(&mut self) {
        if let Some(selected_controller) = self.controller_state.selected() {
            let controller = &self.controllers[selected_controller];
            if !controller.paired_devices.is_empty() {
                self.paired_devices_state.select(Some(0));
            } else {
                self.paired_devices_state.select(None);
            }

            if !controller.new_devices.is_empty() {
                self.new_devices_state.select(Some(0));
            } else {
                self.new_devices_state.select(None);
            }
        }
    }

    pub fn render_set_alias(&mut self, frame: &mut Frame) {
        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Length(6),
                    Constraint::Percentage(45),
                ]
                .as_ref(),
            )
            .split(frame.area());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length((frame.area().width - 80) / 2),
                    Constraint::Min(80),
                    Constraint::Length((frame.area().width - 80) / 2),
                ]
                .as_ref(),
            )
            .split(area[1]);

        let area = area[1];

        let (text_area, alias_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(area);

            let area1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Fill(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            let area2 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
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

        frame.render_widget(Clear, area);
        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .style(Style::default().green())
                .border_style(Style::default().fg(Color::Green)),
            area,
        );

        if let Some(selected_controller) = self.controller_state.selected() {
            let controller = &self.controllers[selected_controller];
            if let Some(index) = self.paired_devices_state.selected() {
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

                let alias = Paragraph::new(self.new_alias.value())
                    .alignment(Alignment::Left)
                    .style(Style::default().fg(Color::White))
                    .block(
                        Block::new()
                            .bg(Color::DarkGray)
                            .padding(Padding::horizontal(2)),
                    );

                frame.render_widget(msg, text_area);
                frame.render_widget(alias, alias_area);
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if let Some(selected_controller_index) = self.controller_state.selected() {
            let selected_controller = &self.controllers[selected_controller_index];
            // Layout
            let render_new_devices = !selected_controller.new_devices.is_empty()
                | selected_controller.is_scanning.load(Ordering::Relaxed);
            let (controller_block, paired_devices_block, new_devices_block) = {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(if render_new_devices {
                        &[
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                        ]
                    } else {
                        &[
                            Constraint::Percentage(50),
                            Constraint::Percentage(50),
                            Constraint::Fill(1),
                        ]
                    })
                    .margin(1)
                    .split(frame.area());
                (chunks[0], chunks[1], chunks[2])
            };

            //Adapters
            let rows: Vec<Row> = self
                .controllers
                .iter()
                .map(|controller| {
                    Row::new(vec![
                        {
                            if selected_controller.name == controller.name {
                                " ".to_string()
                            } else {
                                String::from(" ")
                            }
                        },
                        controller.name.to_string(),
                        controller.alias.to_string(),
                        {
                            if controller.is_powered {
                                "On".to_string()
                            } else {
                                "Off".to_string()
                            }
                        },
                        controller.is_pairable.to_string(),
                        controller.is_discoverable.to_string(),
                    ])
                    .style(if self.focused_block == FocusedBlock::Adapter {
                        if controller.name == selected_controller.name {
                            Style::default().bg(Color::DarkGray)
                        } else {
                            Style::default()
                        }
                    } else {
                        Style::default()
                    })
                })
                .collect();

            let widths = [
                Constraint::Length(4),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(14),
            ];

            let controller_table = Table::new(rows, widths)
                .header({
                    if self.focused_block == FocusedBlock::Adapter {
                        Row::new(vec![
                            Cell::from(""),
                            Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Alias").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Power").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Pairable").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Discoverable").style(Style::default().fg(Color::Yellow)),
                        ])
                        .style(Style::new().bold())
                        .bottom_margin(1)
                    } else {
                        Row::new(vec![
                            Cell::from(""),
                            Cell::from("Name").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                            Cell::from("Alias").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                            Cell::from("Power").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                            Cell::from("Pairable").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                            Cell::from("Discoverable").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                        ])
                        .style(Style::new().bold())
                        .bottom_margin(1)
                    }
                })
                .block(
                    Block::default()
                        .title(" Adapter ")
                        .title_style({
                            if self.focused_block == FocusedBlock::Adapter {
                                Style::default().bold()
                            } else {
                                Style::default()
                            }
                        })
                        .borders(Borders::ALL)
                        .border_style({
                            if self.focused_block == FocusedBlock::Adapter {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default()
                            }
                        })
                        .border_type({
                            if self.focused_block == FocusedBlock::Adapter {
                                BorderType::Thick
                            } else {
                                BorderType::default()
                            }
                        }),
                )
                .style(match self.color_mode {
                    ColorMode::Dark => Style::default().fg(Color::White),
                    ColorMode::Light => Style::default().fg(Color::Black),
                })
                .highlight_style(if self.focused_block == FocusedBlock::Adapter {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                });

            frame.render_widget(controller_table, controller_block);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state =
                ScrollbarState::new(self.controllers.len()).position(selected_controller_index);
            frame.render_stateful_widget(
                scrollbar,
                controller_block.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );

            //Paired devices
            let rows: Vec<Row> = selected_controller
                .paired_devices
                .iter()
                .map(|d| {
                    Row::new(vec![
                        {
                            if let Some(icon) = &d.icon {
                                format!("{} {}", icon, &d.alias)
                            } else {
                                d.alias.to_owned()
                            }
                        },
                        d.is_trusted.to_string(),
                        d.is_connected.to_string(),
                        {
                            if let Some(battery_percentage) = d.battery_percentage {
                                match battery_percentage {
                                    n if n >= 90 => {
                                        format!("{}% 󰥈 ", battery_percentage)
                                    }
                                    n if (80..90).contains(&n) => {
                                        format!("{}% 󰥅 ", battery_percentage)
                                    }
                                    n if (70..80).contains(&n) => {
                                        format!("{}% 󰥄 ", battery_percentage)
                                    }
                                    n if (60..70).contains(&n) => {
                                        format!("{}% 󰥃 ", battery_percentage)
                                    }
                                    n if (50..60).contains(&n) => {
                                        format!("{}% 󰥂 ", battery_percentage)
                                    }
                                    n if (40..50).contains(&n) => {
                                        format!("{}% 󰥁 ", battery_percentage)
                                    }
                                    n if (30..40).contains(&n) => {
                                        format!("{}% 󰥀 ", battery_percentage)
                                    }
                                    n if (20..30).contains(&n) => {
                                        format!("{}% 󰤿 ", battery_percentage)
                                    }
                                    n if (10..20).contains(&n) => {
                                        format!("{}% 󰤾 ", battery_percentage)
                                    }
                                    _ => {
                                        format!("{}% 󰤾 ", battery_percentage)
                                    }
                                }
                            } else {
                                String::new()
                            }
                        },
                    ])
                })
                .collect();
            let rows_len = rows.len();

            let show_battery_column = selected_controller
                .paired_devices
                .iter()
                .any(|device| device.battery_percentage.is_some());

            let mut widths = vec![
                Constraint::Max(25),
                Constraint::Length(10),
                Constraint::Length(10),
            ];

            if show_battery_column {
                widths.push(Constraint::Length(10));
            }

            let paired_devices_table = Table::new(rows, widths)
                .header({
                    if show_battery_column {
                        if self.focused_block == FocusedBlock::PairedDevices {
                            Row::new(vec![
                                Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Trusted").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Connected").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Battery").style(Style::default().fg(Color::Yellow)),
                            ])
                            .style(Style::new().bold())
                            .bottom_margin(1)
                        } else {
                            Row::new(vec![
                                Cell::from("Name").style(match self.color_mode {
                                    ColorMode::Dark => Style::default().fg(Color::White),
                                    ColorMode::Light => Style::default().fg(Color::Black),
                                }),
                                Cell::from("Trusted").style(match self.color_mode {
                                    ColorMode::Dark => Style::default().fg(Color::White),
                                    ColorMode::Light => Style::default().fg(Color::Black),
                                }),
                                Cell::from("Connected").style(match self.color_mode {
                                    ColorMode::Dark => Style::default().fg(Color::White),
                                    ColorMode::Light => Style::default().fg(Color::Black),
                                }),
                                Cell::from("Battery").style(match self.color_mode {
                                    ColorMode::Dark => Style::default().fg(Color::White),
                                    ColorMode::Light => Style::default().fg(Color::Black),
                                }),
                            ])
                            .style(Style::new().bold())
                            .bottom_margin(1)
                        }
                    } else if self.focused_block == FocusedBlock::PairedDevices {
                        Row::new(vec![
                            Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Trusted").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Connected").style(Style::default().fg(Color::Yellow)),
                        ])
                        .style(Style::new().bold())
                        .bottom_margin(1)
                    } else {
                        Row::new(vec![
                            Cell::from("Name").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                            Cell::from("Trusted").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                            Cell::from("Connected").style(match self.color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            }),
                        ])
                        .style(Style::new().bold())
                        .bottom_margin(1)
                    }
                })
                .block(
                    Block::default()
                        .title(" Paired Devices ")
                        .title_style({
                            if self.focused_block == FocusedBlock::PairedDevices {
                                Style::default().bold()
                            } else {
                                Style::default()
                            }
                        })
                        .borders(Borders::ALL)
                        .border_style({
                            if self.focused_block == FocusedBlock::PairedDevices {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default()
                            }
                        })
                        .border_type({
                            if self.focused_block == FocusedBlock::PairedDevices {
                                BorderType::Thick
                            } else {
                                BorderType::default()
                            }
                        }),
                )
                .style(match self.color_mode {
                    ColorMode::Dark => Style::default().fg(Color::White),
                    ColorMode::Light => Style::default().fg(Color::Black),
                })
                .highlight_style(if self.focused_block == FocusedBlock::PairedDevices {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                });

            frame.render_stateful_widget(
                paired_devices_table,
                paired_devices_block,
                &mut self.paired_devices_state.clone(),
            );

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state = ScrollbarState::new(rows_len)
                .position(self.paired_devices_state.selected().unwrap_or_default());
            frame.render_stateful_widget(
                scrollbar,
                paired_devices_block.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );

            //New devices

            if render_new_devices {
                let rows: Vec<Row> = selected_controller
                    .new_devices
                    .iter()
                    .map(|d| {
                        Row::new(vec![d.addr.to_string(), {
                            if let Some(icon) = &d.icon {
                                format!("{} {}", icon, &d.alias)
                            } else {
                                d.alias.to_owned()
                            }
                        }])
                    })
                    .collect();
                let rows_len = rows.len();

                let widths = [Constraint::Length(25), Constraint::Length(20)];

                let new_devices_table = Table::new(rows, widths)
                    .header({
                        if self.focused_block == FocusedBlock::NewDevices {
                            Row::new(vec![
                                Cell::from("Address").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                            ])
                            .style(Style::new().bold())
                            .bottom_margin(1)
                        } else {
                            Row::new(vec![
                                Cell::from("Address").style(match self.color_mode {
                                    ColorMode::Dark => Style::default().fg(Color::White),
                                    ColorMode::Light => Style::default().fg(Color::Black),
                                }),
                                Cell::from("Name").style(match self.color_mode {
                                    ColorMode::Dark => Style::default().fg(Color::White),
                                    ColorMode::Light => Style::default().fg(Color::Black),
                                }),
                            ])
                            .style(Style::new().bold())
                            .bottom_margin(1)
                        }
                    })
                    .block(
                        Block::default()
                            .title({
                                if selected_controller.is_scanning.load(Ordering::Relaxed) {
                                    format!(" Scanning {} ", self.spinner.draw())
                                } else {
                                    String::from(" Discovered devices ")
                                }
                            })
                            .title_style({
                                if self.focused_block == FocusedBlock::NewDevices {
                                    Style::default().bold()
                                } else {
                                    Style::default()
                                }
                            })
                            .borders(Borders::ALL)
                            .border_style({
                                if self.focused_block == FocusedBlock::NewDevices {
                                    Style::default().fg(Color::Green)
                                } else {
                                    Style::default()
                                }
                            })
                            .border_type({
                                if self.focused_block == FocusedBlock::NewDevices {
                                    BorderType::Thick
                                } else {
                                    BorderType::default()
                                }
                            }),
                    )
                    .style(match self.color_mode {
                        ColorMode::Dark => Style::default().fg(Color::White),
                        ColorMode::Light => Style::default().fg(Color::Black),
                    })
                    .highlight_style(if self.focused_block == FocusedBlock::NewDevices {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    });

                let mut state = self.new_devices_state.clone();
                if self.focused_block == FocusedBlock::NewDevices && state.selected().is_none() {
                    state.select(Some(0));
                }

                frame.render_stateful_widget(new_devices_table, new_devices_block, &mut state);

                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓"));
                let mut scrollbar_state =
                    ScrollbarState::new(rows_len).position(state.selected().unwrap_or_default());
                frame.render_stateful_widget(
                    scrollbar,
                    new_devices_block.inner(Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    &mut scrollbar_state,
                );
            }

            // Pairing confirmation

            if self.pairing_confirmation.display.load(Ordering::Relaxed) {
                self.focused_block = FocusedBlock::PassKeyConfirmation;
                if self.pairing_confirmation.message.is_none() {
                    let msg = self
                        .pairing_confirmation
                        .confirmation_message_receiver
                        .recv()
                        .unwrap();
                    self.pairing_confirmation.message = Some(msg);
                }

                let popup_area = popup(frame.area());

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
                        .split(popup_area);

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

                let text = Text::from(
                    self.pairing_confirmation
                        .message
                        .clone()
                        .unwrap_or_default(),
                )
                .style(Style::default().fg(Color::White));
                let (yes, no) = {
                    if self.pairing_confirmation.confirmed {
                        let no = Span::from("[No]").style(Style::default());
                        let yes = Span::from("[Yes]").style(Style::default().bg(Color::DarkGray));
                        (yes, no)
                    } else {
                        let no = Span::from("[No]").style(Style::default().bg(Color::DarkGray));
                        let yes = Span::from("[Yes]").style(Style::default());
                        (yes, no)
                    }
                };

                frame.render_widget(Clear, popup_area);

                frame.render_widget(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Green)),
                    popup_area,
                );
                frame.render_widget(text.alignment(Alignment::Center), text_area);
                frame.render_widget(yes, yes_area);
                frame.render_widget(no, no_area);
            }
        }
    }
    pub async fn new(config: Arc<Config>) -> AppResult<Self> {
        let color_mode = match terminal_light::luma() {
            Ok(luma) if luma > 0.6 => ColorMode::Light,
            Ok(_) => ColorMode::Dark,
            Err(_) => ColorMode::Dark,
        };

        let session = Arc::new(bluer::Session::new().await?);

        // Pairing confirmation
        let pairing_confirmation = PairingConfirmation::default();

        let user_confirmation_receiver = pairing_confirmation.user_confirmation_receiver.clone();

        let confirmation_message_sender = pairing_confirmation.confirmation_message_sender.clone();

        let confirmation_display = pairing_confirmation.display.clone();

        let agent = Agent {
            request_default: false,
            request_confirmation: Some(Box::new(move |req| {
                request_confirmation(
                    req,
                    confirmation_display.clone(),
                    user_confirmation_receiver.clone(),
                    confirmation_message_sender.clone(),
                )
                .boxed()
            })),
            ..Default::default()
        };

        let handle = session.register_agent(agent).await?;
        let controllers: Vec<Controller> = Controller::get_all(session.clone()).await?;

        let mut controller_state = TableState::default();
        if controllers.is_empty() {
            controller_state.select(None);
        } else {
            controller_state.select(Some(0));
        }

        Ok(Self {
            running: true,
            session,
            agent: handle,
            help: Help::new(config),
            spinner: Spinner::default(),
            notifications: Vec::new(),
            controllers,
            controller_state,
            paired_devices_state: TableState::default(),
            new_devices_state: TableState::default(),
            focused_block: FocusedBlock::Adapter,
            pairing_confirmation,
            color_mode,
            new_alias: Input::default(),
        })
    }

    pub async fn tick(&mut self) -> AppResult<()> {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);

        if self.spinner.active {
            self.spinner.update();
        }
        self.refresh().await?;
        Ok(())
    }

    pub async fn refresh(&mut self) -> AppResult<()> {
        if !self.pairing_confirmation.display.load(Ordering::Relaxed)
            & self.pairing_confirmation.message.is_some()
        {
            self.pairing_confirmation.message = None;
        }

        let refreshed_controllers = Controller::get_all(self.session.clone()).await?;

        let names = {
            let mut names: Vec<String> = Vec::new();

            for controller in self.controllers.iter() {
                if !refreshed_controllers
                    .iter()
                    .any(|c| c.name == controller.name)
                {
                    names.push(controller.name.clone());
                }
            }

            names
        };

        // Remove unplugged adapters
        for name in names {
            self.controllers.retain(|c| c.name != name);

            if !self.controllers.is_empty() {
                let i = match self.controller_state.selected() {
                    Some(i) => {
                        if i > 0 {
                            i - 1
                        } else {
                            0
                        }
                    }
                    None => 0,
                };
                self.controller_state.select(Some(i));
            } else {
                self.controller_state.select(None);
            }
        }

        for refreshed_controller in refreshed_controllers {
            if let Some(controller) = self
                .controllers
                .iter_mut()
                .find(|c| c.name == refreshed_controller.name)
            {
                // Update existing adapters
                controller.alias = refreshed_controller.alias;
                controller.is_powered = refreshed_controller.is_powered;
                controller.is_pairable = refreshed_controller.is_pairable;
                controller.is_discoverable = refreshed_controller.is_discoverable;
                controller.paired_devices = refreshed_controller.paired_devices;
                controller.new_devices = refreshed_controller.new_devices;
            } else {
                // Add new detected adapters
                self.controllers.push(refreshed_controller);
            }
        }

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

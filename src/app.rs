use bluer::{
    agent::{Agent, AgentHandle},
    Session,
};
use futures::FutureExt;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Row, Table, TableState},
    Frame,
};
use std::sync::mpsc::channel;

use crate::{
    bluetooth::{request_confirmation, Controller},
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

use async_channel;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedBlock {
    Adapter,
    PairedDevices,
    NewDevices,
    Help,
    PassKeyConfirmation,
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
    pub fn render(&mut self, frame: &mut Frame) {
        if let Some(selected_controller_index) = self.controller_state.selected() {
            let selected_controller = &self.controllers[selected_controller_index];
            // Layout

            let controllers_block_length = self.controllers.len() as u16 + 4;

            let paired_devices_area_length = selected_controller.paired_devices.len() as u16 + 4;

            let new_devices_area_length = selected_controller.new_devices.len() as u16 + 4;

            let (controller_block, paired_devices_block, new_devices_block) = {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(controllers_block_length),
                            Constraint::Length(paired_devices_area_length),
                            Constraint::Length(new_devices_area_length),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());
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
                .header(
                    Row::new(vec![
                        "",
                        "Name",
                        "Alias",
                        "Power",
                        "Pairable",
                        "Discoverable",
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1),
                )
                .block(
                    Block::default()
                        .title("Adapter")
                        .borders(Borders::ALL)
                        .border_type({
                            if self.focused_block == FocusedBlock::Adapter {
                                BorderType::Thick
                            } else {
                                BorderType::default()
                            }
                        }),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(if self.focused_block == FocusedBlock::Adapter {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                });

            frame.render_widget(controller_table, controller_block);

            //Paired devices
            let rows: Vec<Row> = selected_controller
                .paired_devices
                .iter()
                .map(|d| {
                    Row::new(vec![
                        d.alias.to_owned(),
                        d.is_trusted.to_string(),
                        d.is_connected.to_string(),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Max(25),
                Constraint::Length(10),
                Constraint::Length(10),
            ];

            let paired_devices_table = Table::new(rows, widths)
                .header(
                    Row::new(vec!["Name", "Trusted", "Connected"])
                        .style(Style::new().bold())
                        .bottom_margin(1),
                )
                .block(
                    Block::default()
                        .title("Paired Devices")
                        .borders(Borders::ALL)
                        .border_type({
                            if self.focused_block == FocusedBlock::PairedDevices {
                                BorderType::Thick
                            } else {
                                BorderType::default()
                            }
                        }),
                )
                .style(Style::default().fg(Color::White))
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

            //New devices

            if !selected_controller.new_devices.is_empty()
                | selected_controller.is_scanning.load(Ordering::Relaxed)
            {
                let rows: Vec<Row> = selected_controller
                    .new_devices
                    .iter()
                    .map(|d| Row::new(vec![d.addr.to_string(), d.alias.to_owned()]))
                    .collect();

                let widths = [Constraint::Length(25), Constraint::Length(20)];

                let new_devices_table = Table::new(rows, widths)
                    .header(
                        Row::new(vec!["Address", "Name"])
                            .style(Style::new().bold())
                            .bottom_margin(1),
                    )
                    .block(
                        Block::default()
                            .title({
                                if selected_controller.is_scanning.load(Ordering::Relaxed) {
                                    format!("Scanning {} ", self.spinner.draw())
                                } else {
                                    String::from("Discovered devices")
                                }
                            })
                            .borders(Borders::ALL)
                            .border_type({
                                if self.focused_block == FocusedBlock::NewDevices {
                                    BorderType::Thick
                                } else {
                                    BorderType::default()
                                }
                            }),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(if self.focused_block == FocusedBlock::NewDevices {
                        Style::default().bg(Color::Gray)
                    } else {
                        Style::default()
                    });

                let mut state = self.new_devices_state.clone();
                if self.focused_block == FocusedBlock::NewDevices && state.selected().is_none() {
                    state.select(Some(0));
                }

                frame.render_stateful_widget(new_devices_table, new_devices_block, &mut state);
            }

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

                let popup_area = popup(frame.size());

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
                );
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
                        .border_type(BorderType::Thick),
                    popup_area,
                );
                frame.render_widget(text.alignment(Alignment::Center), text_area);
                frame.render_widget(yes, yes_area);
                frame.render_widget(no, no_area);
            }
        }
    }
    pub async fn new() -> AppResult<Self> {
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
            help: Help::new(),
            spinner: Spinner::default(),
            notifications: Vec::new(),
            controllers,
            controller_state,
            paired_devices_state: TableState::default(),
            new_devices_state: TableState::default(),
            focused_block: FocusedBlock::Adapter,
            pairing_confirmation,
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

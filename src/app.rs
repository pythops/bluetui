use crate::{
    agent::{
        display_passkey, display_pin_code, request_confirmation, request_passkey, request_pin_code,
    },
    event::Event,
    help::Help,
};
use bluer::{
    Address, Session,
    agent::{Agent, AgentHandle},
};
use futures::FutureExt;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::Input;

use crate::{
    agent::AuthAgent,
    bluetooth::Controller,
    config::{Config, Width},
    favorite::{read_favorite_devices_from_disk, save_favorite_devices_to_disk},
    notification::Notification,
    requests::Requests,
    spinner::Spinner,
};
use std::sync::{Arc, atomic::Ordering};

pub type AppResult<T> = anyhow::Result<T>;

const STAR_SYMBOL: &str = "★";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedBlock {
    Adapter,
    PairedDevices,
    NewDevices,
    SetDeviceAliasBox,
    RequestConfirmation,
    EnterPinCode,
    EnterPasskey,
    DisplayPinCode,
    DisplayPasskey,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub session: Arc<Session>,
    pub agent: AgentHandle,
    pub spinner: Spinner,
    pub notifications: Vec<Notification>,
    pub controllers: Vec<Controller>,
    pub controller_state: TableState,
    pub paired_devices_state: TableState,
    pub favorite_devices: Vec<Address>,
    pub new_devices_state: TableState,
    pub focused_block: FocusedBlock,
    pub new_alias: Input,
    pub config: Arc<Config>,
    pub requests: Requests,
    pub auth_agent: AuthAgent,
}

impl App {
    pub async fn new(config: Arc<Config>, sender: UnboundedSender<Event>) -> AppResult<Self> {
        let session = Arc::new(bluer::Session::new().await?);

        let auth_agent = AuthAgent::new(sender.clone());

        let agent = Agent {
            request_default: false,
            request_confirmation: Some(Box::new({
                let auth_agent = auth_agent.clone();
                move |request| request_confirmation(request, auth_agent.clone()).boxed()
            })),
            request_pin_code: Some(Box::new({
                let auth_agent = auth_agent.clone();
                move |request| request_pin_code(request, auth_agent.clone()).boxed()
            })),
            request_passkey: Some(Box::new({
                let auth_agent = auth_agent.clone();
                move |request| request_passkey(request, auth_agent.clone()).boxed()
            })),
            display_pin_code: Some(Box::new({
                let auth_agent = auth_agent.clone();
                move |request| display_pin_code(request, auth_agent.clone()).boxed()
            })),
            display_passkey: Some(Box::new({
                let auth_agent = auth_agent.clone();
                move |request| display_passkey(request, auth_agent.clone()).boxed()
            })),
            ..Default::default()
        };

        let favorite_devices = read_favorite_devices_from_disk().await.unwrap_or_default();

        let handle = session.register_agent(agent).await?;
        let controllers: Vec<Controller> =
            Controller::get_all(session.clone(), &favorite_devices).await?;

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
            spinner: Spinner::default(),
            notifications: Vec::new(),
            controllers,
            controller_state,
            paired_devices_state: TableState::default(),
            favorite_devices,
            new_devices_state: TableState::default(),
            focused_block: FocusedBlock::PairedDevices,
            new_alias: Input::default(),
            config,
            requests: Requests::default(),
            auth_agent,
        })
    }

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

    pub fn area(&self, frame: &Frame) -> Rect {
        match self.config.width {
            Width::Size(v) => {
                if v < frame.area().width {
                    let area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(v), Constraint::Fill(1)])
                        .split(frame.area());

                    area[0]
                } else {
                    frame.area()
                }
            }
            _ => frame.area(),
        }
    }

    pub fn render_set_alias(&mut self, frame: &mut Frame, area: Rect) {
        let block = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(6),
                Constraint::Fill(1),
            ])
            .split(area);

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(70),
                Constraint::Fill(1),
            ])
            .split(block[1])[1];

        let (text_block, alias_block) = {
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
                .split(block);

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

        frame.render_widget(Clear, block);
        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .style(Style::default().green())
                .border_style(Style::default().fg(Color::Green)),
            block,
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

                frame.render_widget(msg, text_block);
                frame.render_widget(alias, alias_block);
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        if let Some(selected_controller_index) = self.controller_state.selected() {
            let selected_controller = &self.controllers[selected_controller_index];
            // Layout
            let render_new_devices = selected_controller.is_scanning.load(Ordering::Relaxed);

            if !render_new_devices && self.focused_block == FocusedBlock::NewDevices {
                self.focused_block = FocusedBlock::PairedDevices;
            }

            let adapter_block_height = self.controllers.len() as u16 + 4;

            let paired_devices_block_height = selected_controller.paired_devices.len() as u16 + 4;

            let (paired_devices_block, new_devices_block, controller_block, help_block) = {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(if render_new_devices {
                        [
                            Constraint::Length(paired_devices_block_height),
                            Constraint::Fill(1),
                            Constraint::Length(adapter_block_height),
                            Constraint::Length(2),
                        ]
                    } else {
                        [
                            Constraint::Fill(1),
                            Constraint::Length(0),
                            Constraint::Length(adapter_block_height),
                            Constraint::Length(2),
                        ]
                    })
                    .margin(1)
                    .split(self.area(frame));
                (chunks[0], chunks[1], chunks[2], chunks[3])
            };

            //Adapters
            let rows: Vec<Row> = self
                .controllers
                .iter()
                .map(|controller| {
                    Row::new(vec![
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
                })
                .collect();

            let widths = [
                Constraint::Length(10),
                Constraint::Length(10),
                Constraint::Length(5),
                Constraint::Length(8),
                Constraint::Length(12),
            ];

            let rows_len = rows.len();

            let controller_table = Table::new(rows, widths)
                .header({
                    if self.focused_block == FocusedBlock::Adapter {
                        Row::new(vec![
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
                            Cell::from("Name"),
                            Cell::from("Alias"),
                            Cell::from("Power"),
                            Cell::from("Pairable"),
                            Cell::from("Discoverable"),
                        ])
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
                .flex(self.config.layout)
                .row_highlight_style(if self.focused_block == FocusedBlock::Adapter {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                });

            frame.render_stateful_widget(
                controller_table,
                controller_block,
                &mut self.controller_state.clone(),
            );

            if rows_len > controller_block.height.saturating_sub(4) as usize {
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
            }

            //Paired devices
            let rows: Vec<Row> = selected_controller
                .paired_devices
                .iter()
                .map(|d| {
                    Row::new(vec![
                        if d.is_favorite {
                            STAR_SYMBOL.to_string()
                        } else {
                            "".to_string()
                        },
                        format!("{} {}", &d.icon, &d.alias),
                        d.is_trusted.to_string(),
                        d.is_connected.to_string(),
                        {
                            if let Some(battery_percentage) = d.battery_percentage {
                                match battery_percentage {
                                    n if n >= 90 => {
                                        format!("{battery_percentage}% 󰥈 ")
                                    }
                                    n if (80..90).contains(&n) => {
                                        format!("{battery_percentage}% 󰥅 ")
                                    }
                                    n if (70..80).contains(&n) => {
                                        format!("{battery_percentage}% 󰥄 ")
                                    }
                                    n if (60..70).contains(&n) => {
                                        format!("{battery_percentage}% 󰥃 ")
                                    }
                                    n if (50..60).contains(&n) => {
                                        format!("{battery_percentage}% 󰥂 ")
                                    }
                                    n if (40..50).contains(&n) => {
                                        format!("{battery_percentage}% 󰥁 ")
                                    }
                                    n if (30..40).contains(&n) => {
                                        format!("{battery_percentage}% 󰥀 ")
                                    }
                                    n if (20..30).contains(&n) => {
                                        format!("{battery_percentage}% 󰤿 ")
                                    }
                                    n if (10..20).contains(&n) => {
                                        format!("{battery_percentage}% 󰤾 ")
                                    }
                                    _ => {
                                        format!("{battery_percentage}% 󰤾 ")
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

            if rows_len > 0
                && self.focused_block == FocusedBlock::PairedDevices
                && self.paired_devices_state.selected().is_none()
            {
                self.paired_devices_state.select(Some(0));
            }

            let show_battery_column = selected_controller
                .paired_devices
                .iter()
                .any(|device| device.battery_percentage.is_some());

            let mut widths = vec![
                Constraint::Length(1),
                Constraint::Max(25),
                Constraint::Length(7),
                Constraint::Length(9),
            ];

            if show_battery_column {
                widths.push(Constraint::Length(10));
            }

            let paired_devices_table = Table::new(rows, widths)
                .header({
                    if show_battery_column {
                        if self.focused_block == FocusedBlock::PairedDevices {
                            Row::new(vec![
                                Cell::from("").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Trusted").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Connected").style(Style::default().fg(Color::Yellow)),
                                Cell::from("Battery").style(Style::default().fg(Color::Yellow)),
                            ])
                            .style(Style::new().bold())
                            .bottom_margin(1)
                        } else {
                            Row::new(vec![
                                Cell::from(""),
                                Cell::from("Name"),
                                Cell::from("Trusted"),
                                Cell::from("Connected"),
                                Cell::from("Battery"),
                            ])
                            .bottom_margin(1)
                        }
                    } else if self.focused_block == FocusedBlock::PairedDevices {
                        Row::new(vec![
                            Cell::from("").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Trusted").style(Style::default().fg(Color::Yellow)),
                            Cell::from("Connected").style(Style::default().fg(Color::Yellow)),
                        ])
                        .style(Style::new().bold())
                        .bottom_margin(1)
                    } else {
                        Row::new(vec![
                            Cell::from(""),
                            Cell::from("Name"),
                            Cell::from("Trusted"),
                            Cell::from("Connected"),
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
                .flex(self.config.layout)
                .row_highlight_style(if self.focused_block == FocusedBlock::PairedDevices {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                });

            frame.render_stateful_widget(
                paired_devices_table,
                paired_devices_block,
                &mut self.paired_devices_state.clone(),
            );

            if rows_len > paired_devices_block.height.saturating_sub(4) as usize {
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
            }

            //New devices

            if render_new_devices {
                let rows: Vec<Row> = selected_controller
                    .new_devices
                    .iter()
                    .map(|d| {
                        Row::new(vec![
                            d.addr.to_string(),
                            format!("{} {}", &d.icon, &d.alias),
                        ])
                    })
                    .collect();
                let rows_len = rows.len();

                let widths = [Constraint::Length(20), Constraint::Length(20)];

                let new_devices_table = Table::new(rows, widths)
                    .header({
                        if self.focused_block == FocusedBlock::NewDevices {
                            Row::new(vec![
                                Cell::from(Line::from("Address").fg(Color::Yellow).centered()),
                                Cell::from(Line::from("Name").fg(Color::Yellow).centered()),
                            ])
                            .style(Style::new().bold())
                            .bottom_margin(1)
                        } else {
                            Row::new(vec![
                                Cell::from(Line::from("Address").centered()),
                                Cell::from(Line::from("Name").centered()),
                            ])
                            .bottom_margin(1)
                        }
                    })
                    .block(
                        Block::default()
                            .padding(Padding::horizontal(1))
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
                    .flex(self.config.layout)
                    .row_highlight_style(if self.focused_block == FocusedBlock::NewDevices {
                        Style::default().bg(Color::DarkGray).fg(Color::White)
                    } else {
                        Style::default()
                    });

                let mut state = self.new_devices_state;
                if self.focused_block == FocusedBlock::NewDevices && state.selected().is_none() {
                    state.select(Some(0));
                }

                frame.render_stateful_widget(new_devices_table, new_devices_block, &mut state);

                if rows_len > new_devices_block.height.saturating_sub(4) as usize {
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓"));
                    let mut scrollbar_state = ScrollbarState::new(rows_len)
                        .position(state.selected().unwrap_or_default());
                    frame.render_stateful_widget(
                        scrollbar,
                        new_devices_block.inner(Margin {
                            vertical: 1,
                            horizontal: 0,
                        }),
                        &mut scrollbar_state,
                    );
                }
            }

            //
            let area = self.area(frame);

            // Help
            Help::render(
                frame,
                area,
                self.focused_block,
                help_block,
                self.config.clone(),
            );

            // Pairing confirmation

            // Set alias popup
            if self.focused_block == FocusedBlock::SetDeviceAliasBox {
                self.render_set_alias(frame, area);
            }

            // Request Confirmation
            if let Some(req) = &self.requests.confirmation {
                req.render(frame, area);
            }

            // Request to enter pin code

            if let Some(req) = &self.requests.enter_pin_code {
                req.render(frame, area);
            }

            // Request passkey
            if let Some(req) = &self.requests.enter_passkey {
                req.render(frame, area);
            }

            // Display Pin Code
            if let Some(req) = &self.requests.display_pin_code {
                req.render(frame, area);
            }

            // Display Passkey
            if let Some(req) = &self.requests.display_passkey {
                req.render(frame, area);
            }
        }
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
        let refreshed_controllers =
            Controller::get_all(self.session.clone(), &self.favorite_devices).await?;

        // Remove unplugged adapters in a single pass
        let mut adapter_removed = false;
        self.controllers.retain(|controller| {
            let should_retain = refreshed_controllers
                .iter()
                .any(|c| c.name == controller.name);

            if !should_retain {
                adapter_removed = true;
            }

            should_retain
        });

        // Update selection after removal
        if adapter_removed {
            if !self.controllers.is_empty() {
                let i = match self.controller_state.selected() {
                    Some(i) => {
                        if i > 0 {
                            (i - 1).min(self.controllers.len() - 1)
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
                // Prepare to check if the paired devices list is shrinking
                let old_paired_count = controller.paired_devices.len();
                let new_paired_count = refreshed_controller.paired_devices.len();

                // Update existing adapters
                controller.alias = refreshed_controller.alias;
                controller.is_powered = refreshed_controller.is_powered;
                controller.is_pairable = refreshed_controller.is_pairable;
                controller.is_discoverable = refreshed_controller.is_discoverable;
                controller.paired_devices = refreshed_controller.paired_devices;
                controller.new_devices = refreshed_controller.new_devices;

                // Update selection if paired devices list shrank
                if new_paired_count < old_paired_count
                    && let Some(selected_index) = self.paired_devices_state.selected()
                {
                    // Check if selected index is now out of bounds and update the index if required
                    if new_paired_count == 0 {
                        self.paired_devices_state.select(None);
                    } else if selected_index >= new_paired_count && new_paired_count > 0 {
                        self.paired_devices_state
                            .select(Some(new_paired_count.saturating_sub(1)));
                    }
                }
            } else {
                // Add new detected adapters
                self.controllers.push(refreshed_controller);
            }
        }

        Ok(())
    }

    pub fn quit(&mut self) {
        // TODO: use env_logger error!()
        let _ = save_favorite_devices_to_disk(&self.favorite_devices);
        self.running = false;
    }
}

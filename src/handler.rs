use std::sync::atomic::Ordering;

use crate::app::FocusedBlock;
use crate::app::{App, AppResult};
use crate::event::Event;
use crate::notification::{Notification, NotificationLevel};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
) -> AppResult<()> {
    match key_event.code {
        // Exit the app
        KeyCode::Char('q') => {
            app.quit();
        }
        KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }

        // Show help
        KeyCode::Char('?') => {
            app.focused_block = FocusedBlock::Help;
        }

        // Discard help popup
        KeyCode::Esc => {
            if app.focused_block == FocusedBlock::Help {
                app.focused_block = FocusedBlock::Adapter;
            }
        }

        // Switch focus
        KeyCode::Tab => match app.focused_block {
            FocusedBlock::Adapter => {
                app.focused_block = FocusedBlock::PairedDevices;
                app.reset_devices_state();
            }
            FocusedBlock::PairedDevices => {
                if let Some(selected_controller) = app.controller_state.selected() {
                    let controller = &app.controllers[selected_controller];
                    if controller.new_devices.is_empty() {
                        app.focused_block = FocusedBlock::Adapter
                    } else {
                        app.focused_block = FocusedBlock::NewDevices
                    }
                }
            }
            FocusedBlock::NewDevices => app.focused_block = FocusedBlock::Adapter,
            FocusedBlock::Help => {}
        },

        // scroll down
        KeyCode::Char('j') | KeyCode::Down => match app.focused_block {
            FocusedBlock::Adapter => {
                if !app.controllers.is_empty() {
                    let i = match app.controller_state.selected() {
                        Some(i) => {
                            if i < app.controllers.len() - 1 {
                                i + 1
                            } else {
                                i
                            }
                        }
                        None => 0,
                    };

                    app.reset_devices_state();
                    app.controller_state.select(Some(i));
                }
            }

            FocusedBlock::PairedDevices => {
                if let Some(selected_controller) = app.controller_state.selected() {
                    let controller = &mut app.controllers[selected_controller];

                    if !controller.paired_devices.is_empty() {
                        let i = match app.paired_devices_state.selected() {
                            Some(i) => {
                                if i < controller.paired_devices.len() - 1 {
                                    i + 1
                                } else {
                                    i
                                }
                            }
                            None => 0,
                        };

                        app.paired_devices_state.select(Some(i));
                    }
                }
            }

            FocusedBlock::NewDevices => {
                if let Some(selected_controller) = app.controller_state.selected() {
                    let controller = &mut app.controllers[selected_controller];

                    if !controller.new_devices.is_empty() {
                        let i = match app.new_devices_state.selected() {
                            Some(i) => {
                                if i < controller.new_devices.len() - 1 {
                                    i + 1
                                } else {
                                    i
                                }
                            }
                            None => 0,
                        };

                        app.new_devices_state.select(Some(i));
                    }
                }
            }

            FocusedBlock::Help => {
                app.help.scroll_down();
            }
        },

        // scroll up
        KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
            FocusedBlock::Adapter => {
                if !app.controllers.is_empty() {
                    let i = match app.controller_state.selected() {
                        Some(i) => {
                            if i > 1 {
                                i - 1
                            } else {
                                0
                            }
                        }
                        None => 0,
                    };

                    app.reset_devices_state();
                    app.controller_state.select(Some(i));
                }
            }

            FocusedBlock::PairedDevices => {
                if let Some(selected_controller) = app.controller_state.selected() {
                    let controller = &mut app.controllers[selected_controller];
                    if !controller.paired_devices.is_empty() {
                        let i = match app.paired_devices_state.selected() {
                            Some(i) => {
                                if i > 1 {
                                    i - 1
                                } else {
                                    0
                                }
                            }
                            None => 0,
                        };
                        app.paired_devices_state.select(Some(i));
                    }
                }
            }

            FocusedBlock::NewDevices => {
                if let Some(selected_controller) = app.controller_state.selected() {
                    let controller = &mut app.controllers[selected_controller];
                    if !controller.new_devices.is_empty() {
                        let i = match app.new_devices_state.selected() {
                            Some(i) => {
                                if i > 1 {
                                    i - 1
                                } else {
                                    0
                                }
                            }
                            None => 0,
                        };
                        app.new_devices_state.select(Some(i));
                    }
                }
            }
            FocusedBlock::Help => {
                app.help.scroll_up();
            }
        },

        // Start/Stop Scan
        KeyCode::Char('s') => {
            if let Some(selected_controller) = app.controller_state.selected() {
                let controller = &app.controllers[selected_controller];

                if controller.is_scanning.load(Ordering::Relaxed) {
                    controller
                        .is_scanning
                        .store(false, std::sync::atomic::Ordering::Relaxed);

                    Notification::send(
                        "Scanning stopped".to_string(),
                        NotificationLevel::Info,
                        sender,
                    )?;

                    app.spinner.active = false;
                } else {
                    controller
                        .is_scanning
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                    app.spinner.active = true;
                    let adapter = controller.adapter.clone();
                    let is_scanning = controller.is_scanning.clone();
                    tokio::spawn(async move {
                        let _ = Notification::send(
                            "Scanning started".to_string(),
                            NotificationLevel::Info,
                            sender.clone(),
                        );

                        match adapter.discover_devices().await {
                            Ok(mut discover) => {
                                while let Some(_evt) = discover.next().await {
                                    if !is_scanning.load(Ordering::Relaxed) {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = Notification::send(
                                    e.to_string(),
                                    NotificationLevel::Error,
                                    sender.clone(),
                                );
                            }
                        }
                    });
                }
            }
        }

        _ => {
            match app.focused_block {
                FocusedBlock::PairedDevices => {
                    match key_event.code {
                        // Unpair
                        KeyCode::Char('u') => {
                            if let Some(selected_controller) = app.controller_state.selected() {
                                let controller = &app.controllers[selected_controller];
                                if let Some(index) = app.paired_devices_state.selected() {
                                    let addr = controller.paired_devices[index].addr;
                                    match controller.adapter.remove_device(addr).await {
                                        Ok(_) => {
                                            Notification::send(
                                                "Device unpaired".to_string(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            )?;
                                        }
                                        Err(e) => {
                                            Notification::send(
                                                e.to_string(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            )?;
                                        }
                                    }
                                }
                            }
                        }

                        // Connect / Disconnect
                        KeyCode::Char(' ') => {
                            if let Some(selected_controller) = app.controller_state.selected() {
                                let controller = &app.controllers[selected_controller];
                                if let Some(index) = app.paired_devices_state.selected() {
                                    let addr = controller.paired_devices[index].addr;
                                    let device = controller.adapter.device(addr)?;
                                    if device.is_connected().await? {
                                        match device.disconnect().await {
                                            Ok(_) => {
                                                Notification::send(
                                                    "Device disconnected".to_string(),
                                                    NotificationLevel::Info,
                                                    sender.clone(),
                                                )?;
                                            }
                                            Err(e) => {
                                                Notification::send(
                                                    e.to_string(),
                                                    NotificationLevel::Error,
                                                    sender.clone(),
                                                )?;
                                            }
                                        }
                                    } else {
                                        match device.connect().await {
                                            Ok(_) => {
                                                Notification::send(
                                                    "Device connected".to_string(),
                                                    NotificationLevel::Info,
                                                    sender.clone(),
                                                )?;
                                            }
                                            Err(e) => {
                                                Notification::send(
                                                    e.to_string(),
                                                    NotificationLevel::Error,
                                                    sender.clone(),
                                                )?;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Trust / Untrust
                        KeyCode::Char('t') => {
                            if let Some(selected_controller) = app.controller_state.selected() {
                                let controller = &app.controllers[selected_controller];
                                if let Some(index) = app.paired_devices_state.selected() {
                                    let addr = controller.paired_devices[index].addr;
                                    let device = controller.adapter.device(addr)?;
                                    if device.is_trusted().await? {
                                        match device.set_trusted(false).await {
                                            Ok(_) => {
                                                Notification::send(
                                                    "Device untrusted".to_string(),
                                                    NotificationLevel::Info,
                                                    sender.clone(),
                                                )?;
                                            }
                                            Err(e) => {
                                                Notification::send(
                                                    e.to_string(),
                                                    NotificationLevel::Error,
                                                    sender.clone(),
                                                )?;
                                            }
                                        }
                                    } else {
                                        match device.set_trusted(true).await {
                                            Ok(_) => {
                                                Notification::send(
                                                    "Device trusted".to_string(),
                                                    NotificationLevel::Info,
                                                    sender.clone(),
                                                )?;
                                            }

                                            Err(e) => {
                                                Notification::send(
                                                    e.to_string(),
                                                    NotificationLevel::Error,
                                                    sender.clone(),
                                                )?;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        _ => {}
                    }
                }

                FocusedBlock::Adapter => {
                    match key_event.code {
                        // toggle pairing
                        KeyCode::Char('p') => {
                            if let Some(selected_controller) = app.controller_state.selected() {
                                let adapter = &app.controllers[selected_controller].adapter;
                                if adapter.is_pairable().await? {
                                    adapter.set_pairable(false).await?;
                                    Notification::send(
                                        "Adpater unpairable".to_string(),
                                        NotificationLevel::Info,
                                        sender.clone(),
                                    )?;
                                } else {
                                    adapter.set_pairable(true).await?;
                                    Notification::send(
                                        "Adpater pairable".to_string(),
                                        NotificationLevel::Info,
                                        sender.clone(),
                                    )?;
                                }
                            }
                        }

                        // toggle power
                        KeyCode::Char('o') => {
                            if let Some(selected_controller) = app.controller_state.selected() {
                                let adapter = &app.controllers[selected_controller].adapter;
                                if adapter.is_powered().await? {
                                    match adapter.set_powered(false).await {
                                        Ok(_) => {
                                            Notification::send(
                                                "Adpater powered off".to_string(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            )?;
                                        }
                                        Err(e) => {
                                            Notification::send(
                                                e.to_string(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            )?;
                                        }
                                    }
                                } else {
                                    match adapter.set_powered(true).await {
                                        Ok(_) => {
                                            Notification::send(
                                                "Adpater powered on".to_string(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            )?;
                                        }
                                        Err(e) => {
                                            Notification::send(
                                                e.to_string(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            )?;
                                        }
                                    }
                                }
                            }
                        }

                        // toggle discovery
                        KeyCode::Char('d') => {
                            if let Some(selected_controller) = app.controller_state.selected() {
                                let adapter = &app.controllers[selected_controller].adapter;
                                if adapter.is_discoverable().await? {
                                    match adapter.set_discoverable(false).await {
                                        Ok(_) => {
                                            Notification::send(
                                                "Adpater undiscoverable on".to_string(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            )?;
                                        }
                                        Err(e) => {
                                            Notification::send(
                                                e.to_string(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            )?;
                                        }
                                    }
                                } else {
                                    match adapter.set_discoverable(true).await {
                                        Ok(_) => {
                                            Notification::send(
                                                "Adpater discoverable on".to_string(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            )?;
                                        }
                                        Err(e) => {
                                            Notification::send(
                                                e.to_string(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            )?;
                                        }
                                    }
                                }
                            }
                        }

                        _ => {}
                    }
                }

                FocusedBlock::NewDevices => {
                    // Pair new device
                    if let KeyCode::Char('p') = key_event.code {
                        if let Some(selected_controller) = app.controller_state.selected() {
                            let controller = &app.controllers[selected_controller];
                            if let Some(index) = app.new_devices_state.selected() {
                                let addr = controller.new_devices[index].addr;
                                let device = controller.adapter.device(addr)?;

                                let device_name = device.alias().await?;
                                Notification::send(
                                    format!("Start pairing with the device\n `{}` ", device_name),
                                    NotificationLevel::Info,
                                    sender.clone(),
                                )?;

                                tokio::spawn(async move {
                                    match device.pair().await {
                                        Ok(_) => {
                                            let _ = Notification::send(
                                                "Device paired".to_string(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            );
                                        }
                                        Err(e) => {
                                            let _ = Notification::send(
                                                e.to_string(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            );
                                        }
                                    }
                                });
                            }
                        }
                    }
                }

                FocusedBlock::Help => {}
            }
        }
    }

    Ok(())
}

use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::app::FocusedBlock;
use crate::app::{App, AppResult};
use crate::config::Config;
use crate::event::Event;
use crate::notification::{Notification, NotificationLevel};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use tui_input::backend::crossterm::EventHandler;

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
    config: Arc<Config>,
) -> AppResult<()> {
    match app.focused_block {
        FocusedBlock::SetDeviceAliasBox => match key_event.code {
            KeyCode::Enter => {
                if let Some(selected_controller) = app.controller_state.selected() {
                    let controller = &app.controllers[selected_controller];
                    if let Some(index) = app.paired_devices_state.selected() {
                        let device = &controller.paired_devices[index];
                        match device.set_alias(app.new_alias.value().to_string()).await {
                            Ok(_) => {
                                Notification::send(
                                    "Set New Alias".to_string(),
                                    NotificationLevel::Info,
                                    sender,
                                )?;
                            }
                            Err(e) => {
                                Notification::send(
                                    e.to_string(),
                                    NotificationLevel::Error,
                                    sender,
                                )?;
                            }
                        }
                        app.focused_block = FocusedBlock::PairedDevices;
                        app.new_alias.reset();
                    }
                }
            }

            KeyCode::Esc => {
                app.focused_block = FocusedBlock::PairedDevices;
                app.new_alias.reset();
            }
            _ => {
                app.new_alias
                    .handle_event(&crossterm::event::Event::Key(key_event));
            }
        },
        _ => {
            match key_event.code {
                // Exit the app
                KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                    app.quit();
                }

                KeyCode::Char('q') => {
                    app.quit();
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
                                app.focused_block = FocusedBlock::Adapter;
                            } else {
                                app.focused_block = FocusedBlock::NewDevices;
                            }
                        }
                    }
                    FocusedBlock::NewDevices => app.focused_block = FocusedBlock::Adapter,
                    _ => {}
                },

                KeyCode::BackTab => match app.focused_block {
                    FocusedBlock::Adapter => {
                        if let Some(selected_controller) = app.controller_state.selected() {
                            let controller = &app.controllers[selected_controller];
                            if controller.new_devices.is_empty() {
                                app.focused_block = FocusedBlock::PairedDevices;
                            } else {
                                app.focused_block = FocusedBlock::NewDevices;
                            }
                        }
                    }
                    FocusedBlock::PairedDevices => {
                        app.focused_block = FocusedBlock::Adapter;
                    }
                    FocusedBlock::NewDevices => {
                        app.focused_block = FocusedBlock::PairedDevices;
                        app.reset_devices_state();
                    }
                    _ => {}
                },

                KeyCode::Char('h') => match app.focused_block {
                    FocusedBlock::Adapter => {
                        if let Some(selected_controller) = app.controller_state.selected() {
                            let controller = &app.controllers[selected_controller];
                            if controller.new_devices.is_empty() {
                                app.focused_block = FocusedBlock::PairedDevices;
                            } else {
                                app.focused_block = FocusedBlock::NewDevices;
                            }
                        }
                    }
                    FocusedBlock::PairedDevices => {
                        app.focused_block = FocusedBlock::Adapter;
                    }
                    FocusedBlock::NewDevices => {
                        app.focused_block = FocusedBlock::PairedDevices;
                        app.reset_devices_state();
                    }
                    _ => {}
                },

                KeyCode::Char('l') => match app.focused_block {
                    FocusedBlock::Adapter => {
                        app.focused_block = FocusedBlock::PairedDevices;
                        app.reset_devices_state();
                    }
                    FocusedBlock::PairedDevices => {
                        if let Some(selected_controller) = app.controller_state.selected() {
                            let controller = &app.controllers[selected_controller];
                            if controller.new_devices.is_empty() {
                                app.focused_block = FocusedBlock::Adapter;
                            } else {
                                app.focused_block = FocusedBlock::NewDevices;
                            }
                        }
                    }
                    FocusedBlock::NewDevices => app.focused_block = FocusedBlock::Adapter,
                    _ => {}
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
                    _ => {}
                },

                // scroll up
                KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
                    FocusedBlock::Adapter => {
                        if !app.controllers.is_empty() {
                            let i = match app.controller_state.selected() {
                                Some(i) => i.saturating_sub(1),
                                None => 0,
                            };

                            app.controller_state.select(Some(i));
                        }
                    }

                    FocusedBlock::PairedDevices => {
                        if let Some(selected_controller) = app.controller_state.selected() {
                            let controller = &mut app.controllers[selected_controller];
                            if !controller.paired_devices.is_empty() {
                                let i = match app.paired_devices_state.selected() {
                                    Some(i) => i.saturating_sub(1),
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
                                    Some(i) => i.saturating_sub(1),
                                    None => 0,
                                };
                                app.new_devices_state.select(Some(i));
                            }
                        }
                    }
                    FocusedBlock::Help => {
                        app.help.scroll_up();
                    }
                    _ => {}
                },

                // Start/Stop Scan
                KeyCode::Char(c) if c == config.toggle_scanning => {
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
                                KeyCode::Char(c) if c == config.paired_device.unpair => {
                                    if let Some(selected_controller) =
                                        app.controller_state.selected()
                                    {
                                        let controller = &app.controllers[selected_controller];
                                        if let Some(index) = app.paired_devices_state.selected() {
                                            let addr = controller.paired_devices[index].addr;
                                            match controller.adapter.remove_device(addr).await {
                                                Ok(_) => {
                                                    let _ = Notification::send(
                                                        "Device unpaired".to_string(),
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
                                        }
                                    }
                                }

                                // Connect / Disconnect
                                KeyCode::Char(c) if c == config.paired_device.toggle_connect => {
                                    if let Some(selected_controller) =
                                        app.controller_state.selected()
                                    {
                                        let controller = &app.controllers[selected_controller];
                                        if let Some(index) = app.paired_devices_state.selected() {
                                            let addr = controller.paired_devices[index].addr;
                                            match controller.adapter.device(addr) {
                                                Ok(device) => {
                                                    tokio::spawn(async move {
                                                        match device.is_connected().await {
                                                            Ok(is_connected) => {
                                                                if is_connected {
                                                                    match device.disconnect().await
                                                                    {
                                                                        Ok(_) => {
                                                                            let _ = Notification::send(
                                                                        "Device disconnected"
                                                                            .to_string(),
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
                                                                } else {
                                                                    match device.connect().await {
                                                                        Ok(_) => {
                                                                            let _ = Notification::send(
                                                                        "Device connected"
                                                                            .to_string(),
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
                                                Err(e) => {
                                                    let _ = Notification::send(
                                                        e.to_string(),
                                                        NotificationLevel::Error,
                                                        sender.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                // Trust / Untrust
                                KeyCode::Char(c) if c == config.paired_device.toggle_trust => {
                                    if let Some(selected_controller) =
                                        app.controller_state.selected()
                                    {
                                        let controller = &app.controllers[selected_controller];
                                        if let Some(index) = app.paired_devices_state.selected() {
                                            let addr = controller.paired_devices[index].addr;
                                            match controller.adapter.device(addr) {
                                                Ok(device) => {
                                                    tokio::spawn(async move {
                                                        match device.is_trusted().await {
                                                            Ok(is_trusted) => {
                                                                if is_trusted {
                                                                    match device
                                                                        .set_trusted(false)
                                                                        .await
                                                                    {
                                                                        Ok(_) => {
                                                                            let _ = Notification::send(
                                                                        "Device untrusted"
                                                                            .to_string(),
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
                                                                } else {
                                                                    match device
                                                                        .set_trusted(true)
                                                                        .await
                                                                    {
                                                                        Ok(_) => {
                                                                            let _ = Notification::send(
                                                                        "Device trusted"
                                                                            .to_string(),
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
                                                Err(e) => {
                                                    let _ = Notification::send(
                                                        e.to_string(),
                                                        NotificationLevel::Error,
                                                        sender.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                KeyCode::Char(c) if c == config.paired_device.rename => {
                                    app.focused_block = FocusedBlock::SetDeviceAliasBox;
                                }

                                _ => {}
                            }
                        }

                        FocusedBlock::Adapter => {
                            match key_event.code {
                                // toggle pairing
                                KeyCode::Char(c) if c == config.adapter.toggle_pairing => {
                                    if let Some(selected_controller) =
                                        app.controller_state.selected()
                                    {
                                        let adapter = &app.controllers[selected_controller].adapter;
                                        tokio::spawn({
                                            let adapter = adapter.clone();
                                            async move {
                                                match adapter.is_pairable().await {
                                                    Ok(is_pairable) => {
                                                        if is_pairable {
                                                            match adapter.set_pairable(false).await
                                                            {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter unpairable"
                                                                            .to_string(),
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
                                                        } else {
                                                            match adapter.set_pairable(true).await {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter pairable"
                                                                            .to_string(),
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
                                            }
                                        });
                                    }
                                }

                                // toggle power
                                KeyCode::Char(c) if c == config.adapter.toggle_power => {
                                    if let Some(selected_controller) =
                                        app.controller_state.selected()
                                    {
                                        let adapter = &app.controllers[selected_controller].adapter;
                                        tokio::spawn({
                                            let adapter = adapter.clone();
                                            async move {
                                                match adapter.is_powered().await {
                                                    Ok(is_powered) => {
                                                        if is_powered {
                                                            match adapter.set_powered(false).await {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter powered off"
                                                                            .to_string(),
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
                                                        } else {
                                                            match adapter.set_powered(true).await {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter powered on"
                                                                            .to_string(),
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
                                            }
                                        });
                                    }
                                }

                                // toggle discovery
                                KeyCode::Char(c) if c == config.adapter.toggle_discovery => {
                                    if let Some(selected_controller) =
                                        app.controller_state.selected()
                                    {
                                        let adapter = &app.controllers[selected_controller].adapter;
                                        tokio::spawn({
                                            let adapter = adapter.clone();
                                            async move {
                                                match adapter.is_discoverable().await {
                                                    Ok(is_discoverable) => {
                                                        if is_discoverable {
                                                            match adapter
                                                                .set_discoverable(false)
                                                                .await
                                                            {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter undiscoverable"
                                                                            .to_string(),
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
                                                        } else {
                                                            match adapter
                                                                .set_discoverable(true)
                                                                .await
                                                            {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter discoverable"
                                                                            .to_string(),
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
                                            }
                                        });
                                    }
                                }

                                _ => {}
                            }
                        }

                        FocusedBlock::NewDevices => {
                            // Pair new device
                            if KeyCode::Char(config.new_device.pair) == key_event.code {
                                if let Some(selected_controller) = app.controller_state.selected() {
                                    let controller = &app.controllers[selected_controller];
                                    if let Some(index) = app.new_devices_state.selected() {
                                        let addr = controller.new_devices[index].addr;
                                        match controller.adapter.device(addr) {
                                            Ok(device) => match device.alias().await {
                                                Ok(device_name) => {
                                                    let _ = Notification::send(
                                                        format!(
                                                            "Start pairing with the device\n `{device_name}`",
                                                        ),
                                                        NotificationLevel::Info,
                                                        sender.clone(),
                                                    );

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
                                                Err(e) => {
                                                    let _ = Notification::send(
                                                        e.to_string(),
                                                        NotificationLevel::Error,
                                                        sender.clone(),
                                                    );
                                                }
                                            },
                                            Err(e) => {
                                                let _ = Notification::send(
                                                    e.to_string(),
                                                    NotificationLevel::Error,
                                                    sender.clone(),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        FocusedBlock::PassKeyConfirmation => match key_event.code {
                            KeyCode::Left | KeyCode::Char('h') => {
                                if !app.pairing_confirmation.confirmed {
                                    app.pairing_confirmation.confirmed = true;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                if app.pairing_confirmation.confirmed {
                                    app.pairing_confirmation.confirmed = false;
                                }
                            }

                            KeyCode::Enter => {
                                app.pairing_confirmation
                                    .user_confirmation_sender
                                    .send(app.pairing_confirmation.confirmed)
                                    .await?;
                                app.pairing_confirmation
                                    .display
                                    .store(false, Ordering::Relaxed);
                                app.focused_block = FocusedBlock::PairedDevices;
                                app.pairing_confirmation.message = None;
                            }

                            _ => {}
                        },

                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

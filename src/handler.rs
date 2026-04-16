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

async fn toggle_connect(app: &mut App, sender: UnboundedSender<Event>) {
    if let Some(selected_controller) = app.controller_state.selected() {
        let controller = &app.controllers[selected_controller];
        if let Some(index) = app.paired_devices_state.selected() {
            let paired_device = &controller.paired_devices[index];
            if paired_device.is_busy.load(Ordering::Relaxed) {
                let _ = Notification::send(
                    "Device is busy. Please wait for the current operation to finish.".into(),
                    NotificationLevel::Warning,
                    sender.clone(),
                );
                return;
            }
            paired_device.is_busy.store(true, Ordering::Relaxed);
            match controller.adapter.device(paired_device.addr) {
                Ok(device) => {
                    let paired_device_is_busy = paired_device.is_busy.clone();
                    tokio::spawn(async move {
                        match device.is_connected().await {
                            Ok(is_connected) => {
                                if is_connected {
                                    match device.disconnect().await {
                                        Ok(_) => {
                                            let _ = Notification::send(
                                                "Device disconnected".into(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            );
                                        }
                                        Err(e) => {
                                            let _ = Notification::send(
                                                e.into(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            );
                                        }
                                    }
                                } else {
                                    match device.connect().await {
                                        Ok(_) => {
                                            let _ = Notification::send(
                                                "Device connected".into(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            );
                                        }
                                        Err(e) => {
                                            let _ = Notification::send(
                                                e.into(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = Notification::send(
                                    e.into(),
                                    NotificationLevel::Error,
                                    sender.clone(),
                                );
                            }
                        }
                        paired_device_is_busy.store(false, Ordering::Relaxed);
                    });
                }
                Err(e) => {
                    let _ = Notification::send(e.into(), NotificationLevel::Error, sender.clone());
                }
            }
        }
    }
}

async fn pair(app: &mut App, sender: UnboundedSender<Event>) {
    if let Some(selected_controller) = app.controller_state.selected() {
        let controller = &app.controllers[selected_controller];
        if let Some(index) = app.new_devices_state.selected() {
            let addr = controller.new_devices[index].addr;
            match controller.adapter.device(addr) {
                Ok(device) => match device.alias().await {
                    Ok(device_name) => {
                        let _ = Notification::send(
                            format!("Start pairing with the device {device_name}").into(),
                            NotificationLevel::Info,
                            sender.clone(),
                        );

                        tokio::spawn(async move {
                            match device.pair().await {
                                Ok(_) => {
                                    let _ = Notification::send(
                                        "Device paired".into(),
                                        NotificationLevel::Info,
                                        sender.clone(),
                                    );

                                    let _ = sender.send(Event::NewPairedDevice(device.address()));
                                    match device.set_trusted(true).await {
                                        Ok(_) => {
                                            let _ = Notification::send(
                                                "Device trusted".into(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            );
                                        }
                                        Err(e) => {
                                            let _ = Notification::send(
                                                e.into(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            );
                                        }
                                    };
                                    match device.connect().await {
                                        Ok(_) => {
                                            let _ = Notification::send(
                                                "Device connected".into(),
                                                NotificationLevel::Info,
                                                sender.clone(),
                                            );
                                        }
                                        Err(e) => {
                                            let _ = Notification::send(
                                                e.into(),
                                                NotificationLevel::Error,
                                                sender.clone(),
                                            );
                                        }
                                    };
                                }
                                Err(e) => {
                                    let _ = Notification::send(
                                        e.into(),
                                        NotificationLevel::Error,
                                        sender.clone(),
                                    );
                                    let _ = sender.send(Event::FailedPairing(device.address()));
                                }
                            }
                        });
                    }
                    Err(e) => {
                        let _ =
                            Notification::send(e.into(), NotificationLevel::Error, sender.clone());
                    }
                },
                Err(e) => {
                    let _ = Notification::send(e.into(), NotificationLevel::Error, sender.clone());
                }
            }
        }
    }
}

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
                        match device.set_alias(app.new_alias.value().into()).await {
                            Ok(_) => {
                                Notification::send(
                                    "Set New Alias".into(),
                                    NotificationLevel::Info,
                                    sender,
                                )?;
                            }
                            Err(e) => {
                                Notification::send(e.into(), NotificationLevel::Error, sender)?;
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
        FocusedBlock::RequestConfirmation => match key_event.code {
            KeyCode::Tab => {
                if let Some(confirmation) = &mut app.requests.confirmation {
                    confirmation.toggle_select();
                }
            }
            KeyCode::Esc => {
                if let Some(confirmation) = &mut app.requests.confirmation {
                    confirmation.cancel(&app.auth_agent).await?;
                }
            }
            KeyCode::Enter => {
                if let Some(confirmation) = &mut app.requests.confirmation {
                    confirmation.submit(&app.auth_agent).await?;
                }
            }

            _ => {}
        },
        FocusedBlock::EnterPinCode => {
            if let Some(req) = &mut app.requests.enter_pin_code {
                match key_event.code {
                    KeyCode::Esc => {
                        req.cancel(&app.auth_agent).await?;
                    }

                    _ => {
                        req.handle_key_events(key_event, &app.auth_agent).await?;
                    }
                }
            }
        }
        FocusedBlock::EnterPasskey => {
            if let Some(req) = &mut app.requests.enter_passkey {
                match key_event.code {
                    KeyCode::Esc => {
                        req.cancel(&app.auth_agent).await?;
                    }

                    _ => {
                        req.handle_key_events(key_event, &app.auth_agent).await?;
                    }
                }
            }
        }
        FocusedBlock::DisplayPinCode => {
            if let Some(req) = &mut app.requests.display_pin_code
                && let KeyCode::Esc | KeyCode::Enter = key_event.code
            {
                req.submit(&app.auth_agent).await?;
            }
        }
        FocusedBlock::DisplayPasskey => {
            if let Some(req) = &mut app.requests.display_passkey
                && key_event.code == KeyCode::Esc
            {
                req.cancel(&app.auth_agent).await?;
            }
        }

        _ => {
            match key_event.code {
                // Exit the app
                KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                    app.quit();
                }

                KeyCode::Char('q') => {
                    app.quit();
                }

                KeyCode::Esc if app.config.esc_quit => {
                    app.quit();
                }

                // Switch focus
                KeyCode::Tab | KeyCode::Char('l') => match app.focused_block {
                    FocusedBlock::Adapter => {
                        app.focused_block = FocusedBlock::PairedDevices;
                        app.reset_devices_state();
                    }
                    FocusedBlock::PairedDevices => {
                        if let Some(selected_controller) = app.controller_state.selected() {
                            let controller = &app.controllers[selected_controller];
                            if controller.is_scanning.load(Ordering::Relaxed) {
                                app.focused_block = FocusedBlock::NewDevices;
                            } else {
                                app.focused_block = FocusedBlock::Adapter;
                            }
                        }
                    }
                    FocusedBlock::NewDevices => {
                        app.focused_block = FocusedBlock::Adapter;
                        app.new_devices_state.select(None);
                    }
                    _ => {}
                },

                KeyCode::BackTab | KeyCode::Char('h') => match app.focused_block {
                    FocusedBlock::Adapter => {
                        if let Some(selected_controller) = app.controller_state.selected() {
                            let controller = &app.controllers[selected_controller];
                            if controller.is_scanning.load(Ordering::Relaxed) {
                                app.focused_block = FocusedBlock::NewDevices;
                            } else {
                                app.focused_block = FocusedBlock::PairedDevices;
                            }
                            app.reset_devices_state();
                        }
                    }
                    FocusedBlock::PairedDevices => {
                        app.focused_block = FocusedBlock::Adapter;
                        app.paired_devices_state.select(None);
                    }
                    FocusedBlock::NewDevices => {
                        app.focused_block = FocusedBlock::PairedDevices;
                        app.new_devices_state.select(None);
                    }
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
                                        0
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
                                        if i < controller.new_devices.len() - 1 {
                                            i + 1
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

                    _ => {}
                },

                // scroll up
                KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
                    FocusedBlock::Adapter => {
                        if !app.controllers.is_empty() {
                            let i = match app.controller_state.selected() {
                                Some(i) => {
                                    if i > 0 {
                                        i - 1
                                    } else {
                                        app.controllers.len() - 1
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
                                        if i > 0 {
                                            i - 1
                                        } else {
                                            controller.paired_devices.len() - 1
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
                                        if i > 0 {
                                            i - 1
                                        } else {
                                            controller.new_devices.len() - 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.new_devices_state.select(Some(i));
                            }
                        }
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
                                "Scanning stopped".into(),
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
                                    "Scanning started".into(),
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
                                            e.into(),
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
                                                        "Device unpaired".into(),
                                                        NotificationLevel::Info,
                                                        sender.clone(),
                                                    );
                                                }
                                                Err(e) => {
                                                    let _ = Notification::send(
                                                        e.into(),
                                                        NotificationLevel::Error,
                                                        sender.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                // Connect / Disconnect
                                KeyCode::Enter => toggle_connect(app, sender).await,
                                KeyCode::Char(' ') => toggle_connect(app, sender).await,

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
                                                                            .into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                        }
                                                                        Err(e) => {
                                                                            let _ = Notification::send(
                                                                        e.into(),
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
                                                                            .into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                        }

                                                                        Err(e) => {
                                                                            let _ = Notification::send(
                                                                        e.into(),
                                                                        NotificationLevel::Error,
                                                                        sender.clone(),
                                                                    );
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let _ = Notification::send(
                                                                    e.into(),
                                                                    NotificationLevel::Error,
                                                                    sender.clone(),
                                                                );
                                                            }
                                                        }
                                                    });
                                                }
                                                Err(e) => {
                                                    let _ = Notification::send(
                                                        e.into(),
                                                        NotificationLevel::Error,
                                                        sender.clone(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                // Favorite / Unfavorite
                                KeyCode::Char(c) if c == config.paired_device.toggle_favorite => {
                                    if let Some(selected_controller) =
                                        app.controller_state.selected()
                                    {
                                        let controller = &app.controllers[selected_controller];
                                        if let Some(index) = app.paired_devices_state.selected() {
                                            let address = controller.paired_devices[index].addr;
                                            let _ = sender.send(Event::ToggleFavorite(address));
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
                                                                        "Adapter unpairable".into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                                Err(e) => {
                                                                    let _ = Notification::send(
                                                                        e.into(),
                                                                        NotificationLevel::Error,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                            }
                                                        } else {
                                                            match adapter.set_pairable(true).await {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter pairable".into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                                Err(e) => {
                                                                    let _ = Notification::send(
                                                                        e.into(),
                                                                        NotificationLevel::Error,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let _ = Notification::send(
                                                            e.into(),
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
                                                                            .into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                                Err(e) => {
                                                                    let _ = Notification::send(
                                                                        e.into(),
                                                                        NotificationLevel::Error,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                            }
                                                        } else {
                                                            match adapter.set_powered(true).await {
                                                                Ok(_) => {
                                                                    let _ = Notification::send(
                                                                        "Adapter powered on".into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                                Err(e) => {
                                                                    let _ = Notification::send(
                                                                        e.into(),
                                                                        NotificationLevel::Error,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let _ = Notification::send(
                                                            e.into(),
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
                                                                            .into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                                Err(e) => {
                                                                    let _ = Notification::send(
                                                                        e.into(),
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
                                                                            .into(),
                                                                        NotificationLevel::Info,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                                Err(e) => {
                                                                    let _ = Notification::send(
                                                                        e.into(),
                                                                        NotificationLevel::Error,
                                                                        sender.clone(),
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let _ = Notification::send(
                                                            e.into(),
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
                            match key_event.code {
                                KeyCode::Enter => pair(app, sender).await,
                                KeyCode::Char(' ') => pair(app, sender).await,
                                _ => {}
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

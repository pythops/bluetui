use bluetui::{
    app::{App, AppResult},
    cli,
    config::Config,
    event::{Event, EventHandler},
    handler::handle_key_events,
    rfkill,
    tui::Tui,
};
use clap::Parser;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, process::exit, sync::Arc};

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = cli::Args::parse();

    let config_file_path = args.config_path.map(|config_path| {
        if config_path.exists() {
            config_path.to_owned()
        } else {
            eprintln!("Config file not found");
            exit(1);
        }
    });

    rfkill::check()?;

    let config = Arc::new(Config::new(config_file_path));

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(1_000);
    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    let mut app = App::new(config.clone(), tui.events.sender.clone()).await?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick().await?,
            Event::Key(key_event) => {
                handle_key_events(
                    key_event,
                    &mut app,
                    tui.events.sender.clone(),
                    config.clone(),
                )
                .await?
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
            Event::NewPairedDevice(address) => {
                if let Some(req) = &app.requests.display_passkey
                    && req.device == address
                {
                    app.requests.display_passkey = None;
                }

                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
            }

            Event::ToggleFavorite(address) => {
                if let Some(pos) = app
                    .favorite_devices
                    .iter()
                    .position(|favorite| *favorite == address)
                {
                    app.favorite_devices.swap_remove(pos);
                } else {
                    app.favorite_devices.push(address);
                }
            }

            Event::RequestConfirmation(request) => {
                app.requests.init_confirmation(request);
                app.focused_block = bluetui::app::FocusedBlock::RequestConfirmation;
            }

            Event::ConfirmationSubmitted => {
                app.requests.confirmation = None;
                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
            }

            Event::RequestEnterPinCode(request) => {
                app.requests.init_enter_pin_code(request);
                app.focused_block = bluetui::app::FocusedBlock::EnterPinCode;
            }

            Event::PinCodeSumitted => {
                app.requests.enter_pin_code = None;
                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
            }

            Event::RequestEnterPasskey(request) => {
                app.requests.init_enter_passkey(request);
                app.focused_block = bluetui::app::FocusedBlock::EnterPasskey;
            }

            Event::PasskeySumitted => {
                app.requests.enter_passkey = None;
                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
            }

            Event::RequestDisplayPinCode(request) => {
                app.requests.init_display_pin_code(request);
                app.focused_block = bluetui::app::FocusedBlock::DisplayPinCode;
            }

            Event::DisplayPinCodeSeen => {
                app.requests.display_pin_code = None;
                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
            }

            Event::RequestDisplayPasskey(request) => {
                app.requests.init_display_passkey(request);
                app.focused_block = bluetui::app::FocusedBlock::DisplayPasskey;
            }

            Event::DisplayPasskeyCanceled => {
                app.requests.display_passkey = None;
                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
            }

            Event::FailedPairing(address) => {
                if let Some(req) = &app.requests.display_passkey
                    && req.device == address
                {
                    app.requests.display_passkey = None;
                }

                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
            }

            Event::Mouse(_) | Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}

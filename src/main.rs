use bluetui::{
    app::{App, AppResult},
    cli,
    config::Config,
    event::{Event, EventHandler},
    handler::handle_key_events,
    rfkill,
    tui::Tui,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, path::PathBuf, process::exit, sync::Arc};

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = cli::cli().get_matches();

    let config_file_path = if let Some(config) = args.get_one::<PathBuf>("config") {
        if config.exists() {
            Some(config.to_owned())
        } else {
            eprintln!("Config file not found");
            exit(1);
        }
    } else {
        None
    };

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
            Event::NewPairedDevice => {
                app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
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
                if let Some(req) = &mut app.requests.display_passkey {
                    req.entered = request.entered;
                } else {
                    app.requests.init_display_passkey(request);
                    app.focused_block = bluetui::app::FocusedBlock::DisplayPasskey;
                }
            }
            Event::DisplayPasskeySeen => {
                if let Some(req) = &mut app.requests.display_passkey {
                    if req.entered > 6 {
                        app.requests.display_passkey = None;
                        app.focused_block = bluetui::app::FocusedBlock::PairedDevices;
                    }
                }
                //TODO: handle when the user cancel
            }

            Event::Mouse(_) | Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}

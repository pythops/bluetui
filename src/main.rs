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
    let mut app = App::new(config.clone()).await?;
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(1_000);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

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
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}

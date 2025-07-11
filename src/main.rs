use bluetui::{
    app::{App, AppResult},
    config::Config,
    event::{Event, EventHandler},
    handler::handle_key_events,
    rfkill,
    tui::Tui,
};
use clap::{Command, crate_version};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc};

#[tokio::main]
async fn main() -> AppResult<()> {
    Command::new("bluetui")
        .version(crate_version!())
        .get_matches();

    rfkill::check()?;

    let config = Arc::new(Config::new());
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

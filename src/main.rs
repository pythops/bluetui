use bluetui::app::{App, AppResult};
use bluetui::config::Config;
use bluetui::event::{Event, EventHandler};
use bluetui::handler::handle_key_events;
use bluetui::tui::Tui;
use clap::{crate_version, Command};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    Command::new("bluetui")
        .version(crate_version!())
        .get_matches();

    let config = Arc::new(Config::new());
    let mut app = App::new(config.clone()).await?;
    let backend = CrosstermBackend::new(io::stderr());
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
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
        }
    }

    tui.exit()?;
    Ok(())
}

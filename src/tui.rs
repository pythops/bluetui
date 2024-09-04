use crate::app::{App, AppResult};
use crate::event::EventHandler;
use crate::ui;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use std::panic;

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> AppResult<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> AppResult<()> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    fn reset() -> AppResult<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> AppResult<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

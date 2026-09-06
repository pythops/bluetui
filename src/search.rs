use std::sync::Arc;

use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::Paragraph,
};
use tui_input::Input;

use crate::{
    bluetooth::Device,
    config::{Config, SearchField},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchTarget {
    PairedDevices,
    NewDevices,
}

#[derive(Debug, Clone)]
pub struct Search {
    pub target: SearchTarget,
    pub input: Input,
    pub previous_selection: Option<usize>,
    pub case_sensitive: bool,
    pub fields: Vec<SearchField>,
}

impl Search {
    pub fn new(
        target: SearchTarget,
        previous_selection: Option<usize>,
        case_sensitive: bool,
        fields: Vec<SearchField>,
    ) -> Self {
        Self {
            target,
            input: Input::default(),
            previous_selection,
            case_sensitive,
            fields,
        }
    }

    pub fn query(&self) -> &str {
        self.input.value()
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, config: Arc<Config>) {
        let line = Line::from(vec![
            Span::from(config.search.to_string()).bold(),
            Span::raw(" "),
            Span::from(self.query()).on_dark_gray(),
        ]);

        let paragraph = Paragraph::new(line).centered().blue();
        frame.render_widget(paragraph, area);
    }
}

pub fn name_matches(name: &str, query: &str, case_sensitive: bool) -> bool {
    if query.is_empty() {
        return true;
    }

    if case_sensitive {
        name.contains(query)
    } else {
        name.to_lowercase().contains(&query.to_lowercase())
    }
}

pub fn device_matches(
    device: &Device,
    query: &str,
    case_sensitive: bool,
    fields: &[SearchField],
) -> bool {
    fields.iter().any(|field| match field {
        SearchField::Alias => name_matches(&device.alias, query, case_sensitive),
        SearchField::Address => name_matches(&device.addr.to_string(), query, case_sensitive),
    })
}

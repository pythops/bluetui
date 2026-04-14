use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Theme {
    pub focused_border: Style,
    pub focused_title: Style,
    pub selected_row: Style,
    pub header: Style,
    pub input: Style,
    pub popup_border: Style,
    pub popup_text: Style,
    pub button_active: Style,
    pub button_inactive: Style,
    pub notification_info: Style,
    pub notification_warning: Style,
    pub notification_error: Style,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ThemeFile {
    focused_border: Option<StyleDef>,
    focused_title: Option<StyleDef>,
    selected_row: Option<StyleDef>,
    header: Option<StyleDef>,
    input: Option<StyleDef>,
    popup_border: Option<StyleDef>,
    popup_text: Option<StyleDef>,
    button_active: Option<StyleDef>,
    button_inactive: Option<StyleDef>,
    notification_info: Option<StyleDef>,
    notification_warning: Option<StyleDef>,
    notification_error: Option<StyleDef>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)]
struct StyleDef {
    fg: Option<ColorDef>,
    bg: Option<ColorDef>,
    modifiers: Option<Vec<ModifierDef>>,
}

#[derive(Debug, Clone, Copy)]
struct ColorDef(Color);

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModifierDef {
    Bold,
    Italic,
    Underlined,
    Reversed,
    Dim,
}

impl<'de> Deserialize<'de> for ColorDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        parse_color(&value)
            .map(ColorDef)
            .map_err(serde::de::Error::custom)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            focused_border: Style::default().fg(Color::Green),
            focused_title: Style::default().add_modifier(Modifier::BOLD),
            selected_row: Style::default().fg(Color::White).bg(Color::DarkGray),
            header: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            input: Style::default().fg(Color::White).bg(Color::DarkGray),
            popup_border: Style::default().fg(Color::Green),
            popup_text: Style::default().fg(Color::White),
            button_active: Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            button_inactive: Style::default(),
            notification_info: Style::default().fg(Color::Green),
            notification_warning: Style::default().fg(Color::Yellow),
            notification_error: Style::default().fg(Color::Red),
        }
    }
}

impl Theme {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read theme file {}", path.display()))?;
        let theme_file: ThemeFile = toml::from_str(&content)
            .with_context(|| format!("failed to parse theme file {}", path.display()))?;

        Ok(Self::default().apply(theme_file))
    }

    fn apply(mut self, theme_file: ThemeFile) -> Self {
        if let Some(style) = theme_file.focused_border {
            self.focused_border = style.apply(self.focused_border);
        }
        if let Some(style) = theme_file.focused_title {
            self.focused_title = style.apply(self.focused_title);
        }
        if let Some(style) = theme_file.selected_row {
            self.selected_row = style.apply(self.selected_row);
        }
        if let Some(style) = theme_file.header {
            self.header = style.apply(self.header);
        }
        if let Some(style) = theme_file.input {
            self.input = style.apply(self.input);
        }
        if let Some(style) = theme_file.popup_border {
            self.popup_border = style.apply(self.popup_border);
        }
        if let Some(style) = theme_file.popup_text {
            self.popup_text = style.apply(self.popup_text);
        }
        if let Some(style) = theme_file.button_active {
            self.button_active = style.apply(self.button_active);
        }
        if let Some(style) = theme_file.button_inactive {
            self.button_inactive = style.apply(self.button_inactive);
        }
        if let Some(style) = theme_file.notification_info {
            self.notification_info = style.apply(self.notification_info);
        }
        if let Some(style) = theme_file.notification_warning {
            self.notification_warning = style.apply(self.notification_warning);
        }
        if let Some(style) = theme_file.notification_error {
            self.notification_error = style.apply(self.notification_error);
        }

        self
    }
}

impl StyleDef {
    fn apply(&self, mut base: Style) -> Style {
        if let Some(fg) = self.fg {
            base = base.fg(fg.0);
        }
        if let Some(bg) = self.bg {
            base = base.bg(bg.0);
        }
        if let Some(modifiers) = &self.modifiers {
            for modifier in modifiers {
                base = base.add_modifier(modifier.to_modifier());
            }
        }
        base
    }
}

impl ModifierDef {
    fn to_modifier(self) -> Modifier {
        match self {
            Self::Bold => Modifier::BOLD,
            Self::Italic => Modifier::ITALIC,
            Self::Underlined => Modifier::UNDERLINED,
            Self::Reversed => Modifier::REVERSED,
            Self::Dim => Modifier::DIM,
        }
    }
}

fn parse_color(value: &str) -> Result<Color> {
    let normalized = value.trim().to_ascii_lowercase();
    let color = match normalized.as_str() {
        "reset" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ if normalized.starts_with('#') => return parse_hex_color(&normalized),
        _ => bail!("unknown color value '{value}'"),
    };

    Ok(color)
}

fn parse_hex_color(value: &str) -> Result<Color> {
    if value.len() != 7 {
        bail!("hex colors must use #RRGGBB format, got '{value}'");
    }

    let r = u8::from_str_radix(&value[1..3], 16)
        .with_context(|| format!("invalid red component in '{value}'"))?;
    let g = u8::from_str_radix(&value[3..5], 16)
        .with_context(|| format!("invalid green component in '{value}'"))?;
    let b = u8::from_str_radix(&value[5..7], 16)
        .with_context(|| format!("invalid blue component in '{value}'"))?;

    Ok(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn temp_file_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("bluetui-{name}-{nanos}.toml"))
    }

    #[test]
    fn load_missing_theme_file_falls_back_to_default() {
        let theme = Theme::load(Some(Path::new(
            "/tmp/bluetui-definitely-missing-theme.toml",
        )))
        .unwrap();

        assert_eq!(theme.selected_row.bg, Some(Color::DarkGray));
        assert_eq!(theme.selected_row.fg, Some(Color::White));
    }

    #[test]
    fn load_theme_file_applies_partial_overrides() {
        let path = temp_file_path("theme");
        fs::write(
            &path,
            r##"
[selected_row]
fg = "#111111"
bg = "#eeeeee"
modifiers = ["bold"]

[focused_border]
fg = "blue"
"##,
        )
        .unwrap();

        let theme = Theme::load(Some(&path)).unwrap();

        assert_eq!(theme.selected_row.fg, Some(Color::Rgb(0x11, 0x11, 0x11)));
        assert_eq!(theme.selected_row.bg, Some(Color::Rgb(0xee, 0xee, 0xee)));
        assert!(theme.selected_row.add_modifier.contains(Modifier::BOLD));
        assert_eq!(theme.focused_border.fg, Some(Color::Blue));
        assert_eq!(theme.popup_border.fg, Some(Color::Green));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_theme_file_returns_error() {
        let path = temp_file_path("invalid-theme");
        fs::write(
            &path,
            r#"
[selected_row]
fg = "not-a-color"
"#,
        )
        .unwrap();

        let err = Theme::load(Some(&path)).unwrap_err().to_string();
        assert!(err.contains("failed to parse theme file"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_modifier_returns_error() {
        let path = temp_file_path("invalid-modifier");
        fs::write(
            &path,
            r#"
[header]
modifiers = ["flashy"]
"#,
        )
        .unwrap();

        let err = Theme::load(Some(&path)).unwrap_err().to_string();
        assert!(err.contains("failed to parse theme file"));

        let _ = fs::remove_file(path);
    }
}

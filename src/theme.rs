use ratatui::style::{Color, Style, Stylize};

pub struct AppTheme {
    pub accent: Style,
    pub normal: Style,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            accent: Style::new().fg(Color::LightMagenta).bold(),
            normal: Style::new(),
        }
    }
}

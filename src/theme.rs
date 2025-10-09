use ratatui::style::{Color, Style, Stylize};

pub struct AppTheme {
    pub accent: Style,
    pub border: Style,
    pub job_normal: Style,
    pub job_selected: Style,
    pub keybind_accent: Style,
    pub normal: Style,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            accent: Style::new().fg(Color::LightMagenta).bold(),
            border: Style::new().fg(Color::LightMagenta),
            job_normal: Style::new(),
            job_selected: Style::new().fg(Color::White).bg(Color::Magenta),
            keybind_accent: Style::new().fg(Color::LightBlue).bold(),
            normal: Style::new(),
        }
    }
}

impl From<UserTheme> for AppTheme {
    fn from(user: UserTheme) -> Self {
        let accent = user.accent.unwrap_or_else(Style::new);
        let accent_bg = accent.bg.unwrap_or(Color::Reset);
        let accent_fg = accent.fg.unwrap_or(Color::Reset);

        Self {
            accent,
            border: user.border.unwrap_or_else(|| {
                accent
                    .not_italic()
                    .not_bold()
                    .not_underlined()
                    .bg(Color::Reset)
            }),
            job_normal: user.job_normal.unwrap_or_else(Style::new),
            job_selected: user
                .job_selected
                .unwrap_or_else(|| accent.fg(accent_bg).bg(accent_fg)),
            keybind_accent: user.keybind_accent.unwrap_or(accent),
            normal: user.normal.unwrap_or_else(Style::new),
        }
    }
}

#[derive(Default)]
pub struct UserTheme {
    pub accent: Option<Style>,
    pub border: Option<Style>,
    pub job_normal: Option<Style>,
    pub job_selected: Option<Style>,
    pub keybind_accent: Option<Style>,
    pub normal: Option<Style>,
}

impl UserTheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accent(mut self, style: impl Into<Style>) -> Self {
        self.accent = Some(style.into());
        self
    }
}

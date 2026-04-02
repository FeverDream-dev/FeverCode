pub mod colors;

use ratatui::style::{Color, Modifier, Style};

pub use colors::Palette;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub palette: Palette,
}

impl Theme {
    fn new(name: &'static str, palette: Palette) -> Self {
        Self { name, palette }
    }

    pub fn fever_dream() -> Self {
        Self::new("fever-dream", Palette::fever_dream())
    }

    pub fn fever_obsidian() -> Self {
        Self::new("fever-obsidian", Palette::fever_obsidian())
    }

    pub fn fever_sandstone() -> Self {
        Self::new("fever-sandstone", Palette::fever_sandstone())
    }

    pub fn fever_papyrus() -> Self {
        Self::new("fever-papyrus", Palette::fever_papyrus())
    }

    pub fn fever_midnight() -> Self {
        Self::new("fever-midnight", Palette::fever_midnight())
    }

    pub fn minimal_mono() -> Self {
        Self::new("minimal-mono", Palette::minimal_mono())
    }

    pub fn terminal_native() -> Self {
        Self::new("terminal-native", Palette::terminal_native())
    }

    pub fn solarized_dark() -> Self {
        Self::new("solarized-dark", Palette::solarized_dark())
    }

    pub fn gruvbox_dark() -> Self {
        Self::new("gruvbox-dark", Palette::gruvbox_dark())
    }

    pub fn high_contrast() -> Self {
        Self::new("high-contrast", Palette::high_contrast())
    }

    pub fn fallback_16() -> Self {
        Self::new("fallback-16", Palette::fallback_16())
    }

    pub fn detect() -> Self {
        if Self::supports_truecolor() {
            Self::fever_dream()
        } else {
            Self::fallback_16()
        }
    }

    fn supports_truecolor() -> bool {
        std::env::var("COLORTERM")
            .map(|v| v.contains("truecolor") || v.contains("24bit"))
            .unwrap_or(false)
    }

    pub fn list_all() -> Vec<Theme> {
        vec![
            Self::fever_dream(),
            Self::fever_obsidian(),
            Self::fever_sandstone(),
            Self::fever_papyrus(),
            Self::fever_midnight(),
            Self::minimal_mono(),
            Self::terminal_native(),
            Self::solarized_dark(),
            Self::gruvbox_dark(),
            Self::high_contrast(),
            Self::fallback_16(),
        ]
    }

    pub fn find_by_name(name: &str) -> Option<Self> {
        Self::list_all().into_iter().find(|t| t.name == name)
    }

    pub fn bg(&self) -> Color {
        self.palette.bg
    }

    pub fn bg_secondary(&self) -> Color {
        self.palette.bg_secondary
    }

    pub fn bg_panel(&self) -> Color {
        self.palette.bg_panel
    }

    pub fn fg(&self) -> Color {
        self.palette.fg
    }

    pub fn fg_secondary(&self) -> Color {
        self.palette.fg_secondary
    }

    pub fn fg_dimmed(&self) -> Color {
        self.palette.fg_dimmed
    }

    pub fn accent(&self) -> Color {
        self.palette.accent
    }

    pub fn success(&self) -> Color {
        self.palette.success
    }

    pub fn warning(&self) -> Color {
        self.palette.warning
    }

    pub fn error(&self) -> Color {
        self.palette.error
    }

    pub fn gold(&self) -> Color {
        self.palette.gold
    }

    pub fn border_color(&self) -> Color {
        self.palette.border
    }

    pub fn style_fg(&self) -> Style {
        Style::default().fg(self.fg())
    }

    pub fn style_dimmed(&self) -> Style {
        Style::default().fg(self.fg_dimmed())
    }

    pub fn style_accent(&self) -> Style {
        Style::default().fg(self.accent())
    }

    pub fn style_accent_bold(&self) -> Style {
        Style::default()
            .fg(self.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn style_success(&self) -> Style {
        Style::default().fg(self.success())
    }

    pub fn style_error(&self) -> Style {
        Style::default().fg(self.error())
    }

    pub fn style_warning(&self) -> Style {
        Style::default().fg(self.warning())
    }

    pub fn style_title(&self, focused: bool) -> Style {
        if focused {
            Style::default()
                .fg(self.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.fg_secondary())
        }
    }

    pub fn style_border(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.accent())
        } else {
            Style::default().fg(self.border_color())
        }
    }
}

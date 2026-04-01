use ratatui::style::{Color, Modifier, Style};

use super::colors::Palette;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: &'static str,
    pub palette: Palette,
}

impl Theme {
    pub fn cold_sacred() -> Self {
        Self {
            name: "cold_sacred",
            palette: Palette::cold_sacred(),
        }
    }

    pub fn cold_sacred_16() -> Self {
        Self {
            name: "cold_sacred_16",
            palette: Palette::cold_sacred_16(),
        }
    }

    pub fn detect() -> Self {
        if Self::supports_truecolor() {
            Self::cold_sacred()
        } else {
            Self::cold_sacred_16()
        }
    }

    fn supports_truecolor() -> bool {
        std::env::var("COLORTERM")
            .map(|v| v.contains("truecolor") || v.contains("24bit"))
            .unwrap_or(false)
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
            Style::default().fg(self.fg_dimmed())
        }
    }
}

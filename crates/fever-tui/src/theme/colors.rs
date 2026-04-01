use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub bg_secondary: Color,
    pub bg_panel: Color,
    pub fg: Color,
    pub fg_secondary: Color,
    pub fg_dimmed: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub info: Color,
    pub highlight: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub gold: Color,
}

impl Palette {
    pub fn cold_sacred() -> Self {
        Self {
            bg: Color::Rgb(10, 10, 15),
            bg_secondary: Color::Rgb(18, 18, 26),
            bg_panel: Color::Rgb(26, 26, 46),
            fg: Color::Rgb(224, 224, 232),
            fg_secondary: Color::Rgb(160, 160, 176),
            fg_dimmed: Color::Rgb(96, 96, 112),
            accent: Color::Rgb(0, 229, 255),
            accent_secondary: Color::Rgb(79, 195, 247),
            info: Color::Rgb(26, 35, 126),
            highlight: Color::Rgb(197, 202, 233),
            success: Color::Rgb(105, 240, 174),
            warning: Color::Rgb(255, 171, 64),
            error: Color::Rgb(255, 23, 68),
            gold: Color::Rgb(255, 213, 79),
        }
    }

    pub fn cold_sacred_16() -> Self {
        Self {
            bg: Color::Black,
            bg_secondary: Color::Black,
            bg_panel: Color::Black,
            fg: Color::White,
            fg_secondary: Color::Gray,
            fg_dimmed: Color::DarkGray,
            accent: Color::Cyan,
            accent_secondary: Color::Cyan,
            info: Color::Blue,
            highlight: Color::White,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            gold: Color::Yellow,
        }
    }
}

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
    pub border: Color,
}

impl Palette {
    // ── Fever Dream themes ──────────────────────────────────────────

    /// Default: deep blue-black with cyan accents — "cold sacred" aesthetic
    pub fn fever_dream() -> Self {
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
            border: Color::Rgb(40, 40, 60),
        }
    }

    /// Obsidian: near-black with warm amber/gold accents — temple stone
    pub fn fever_obsidian() -> Self {
        Self {
            bg: Color::Rgb(8, 8, 10),
            bg_secondary: Color::Rgb(16, 14, 18),
            bg_panel: Color::Rgb(28, 24, 20),
            fg: Color::Rgb(220, 210, 195),
            fg_secondary: Color::Rgb(170, 155, 135),
            fg_dimmed: Color::Rgb(100, 88, 72),
            accent: Color::Rgb(218, 165, 32),
            accent_secondary: Color::Rgb(180, 130, 20),
            info: Color::Rgb(60, 45, 20),
            highlight: Color::Rgb(240, 200, 120),
            success: Color::Rgb(120, 200, 80),
            warning: Color::Rgb(230, 160, 50),
            error: Color::Rgb(220, 60, 60),
            gold: Color::Rgb(255, 195, 0),
            border: Color::Rgb(50, 40, 30),
        }
    }

    /// Sandstone: warm beige/cream with terracotta accents — desert tomb
    pub fn fever_sandstone() -> Self {
        Self {
            bg: Color::Rgb(30, 25, 20),
            bg_secondary: Color::Rgb(42, 36, 30),
            bg_panel: Color::Rgb(55, 46, 38),
            fg: Color::Rgb(235, 220, 200),
            fg_secondary: Color::Rgb(190, 170, 148),
            fg_dimmed: Color::Rgb(130, 112, 90),
            accent: Color::Rgb(200, 120, 60),
            accent_secondary: Color::Rgb(170, 100, 50),
            info: Color::Rgb(80, 55, 30),
            highlight: Color::Rgb(230, 180, 120),
            success: Color::Rgb(130, 190, 90),
            warning: Color::Rgb(220, 170, 60),
            error: Color::Rgb(200, 70, 60),
            gold: Color::Rgb(210, 170, 50),
            border: Color::Rgb(70, 58, 45),
        }
    }

    /// Papyrus: aged parchment with deep ink accents — scroll/ancient text
    pub fn fever_papyrus() -> Self {
        Self {
            bg: Color::Rgb(38, 34, 28),
            bg_secondary: Color::Rgb(48, 42, 34),
            bg_panel: Color::Rgb(60, 52, 40),
            fg: Color::Rgb(200, 190, 170),
            fg_secondary: Color::Rgb(160, 148, 128),
            fg_dimmed: Color::Rgb(110, 100, 80),
            accent: Color::Rgb(140, 90, 50),
            accent_secondary: Color::Rgb(120, 75, 40),
            info: Color::Rgb(70, 50, 30),
            highlight: Color::Rgb(200, 170, 120),
            success: Color::Rgb(110, 160, 80),
            warning: Color::Rgb(190, 140, 50),
            error: Color::Rgb(180, 60, 50),
            gold: Color::Rgb(180, 150, 50),
            border: Color::Rgb(80, 68, 50),
        }
    }

    /// Midnight: deep navy with electric blue and violet accents — night sky
    pub fn fever_midnight() -> Self {
        Self {
            bg: Color::Rgb(6, 6, 22),
            bg_secondary: Color::Rgb(12, 12, 36),
            bg_panel: Color::Rgb(22, 18, 52),
            fg: Color::Rgb(210, 215, 240),
            fg_secondary: Color::Rgb(150, 155, 190),
            fg_dimmed: Color::Rgb(85, 88, 130),
            accent: Color::Rgb(100, 130, 255),
            accent_secondary: Color::Rgb(160, 100, 255),
            info: Color::Rgb(30, 20, 80),
            highlight: Color::Rgb(180, 180, 255),
            success: Color::Rgb(80, 220, 180),
            warning: Color::Rgb(255, 190, 80),
            error: Color::Rgb(255, 80, 100),
            gold: Color::Rgb(200, 180, 255),
            border: Color::Rgb(35, 30, 65),
        }
    }

    // ── Functional themes ───────────────────────────────────────────

    /// Minimal Mono: pure black/white — no color distractions
    pub fn minimal_mono() -> Self {
        Self {
            bg: Color::Rgb(0, 0, 0),
            bg_secondary: Color::Rgb(18, 18, 18),
            bg_panel: Color::Rgb(30, 30, 30),
            fg: Color::Rgb(230, 230, 230),
            fg_secondary: Color::Rgb(170, 170, 170),
            fg_dimmed: Color::Rgb(100, 100, 100),
            accent: Color::Rgb(255, 255, 255),
            accent_secondary: Color::Rgb(200, 200, 200),
            info: Color::Rgb(60, 60, 60),
            highlight: Color::Rgb(255, 255, 255),
            success: Color::Rgb(180, 255, 180),
            warning: Color::Rgb(255, 255, 150),
            error: Color::Rgb(255, 150, 150),
            gold: Color::Rgb(200, 200, 200),
            border: Color::Rgb(60, 60, 60),
        }
    }

    /// Terminal Native: uses standard terminal colors
    pub fn terminal_native() -> Self {
        Self {
            bg: Color::Black,
            bg_secondary: Color::Black,
            bg_panel: Color::Rgb(25, 25, 25),
            fg: Color::White,
            fg_secondary: Color::Gray,
            fg_dimmed: Color::DarkGray,
            accent: Color::Cyan,
            accent_secondary: Color::Blue,
            info: Color::Blue,
            highlight: Color::White,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            gold: Color::Yellow,
            border: Color::DarkGray,
        }
    }

    // ── Inspired themes ─────────────────────────────────────────────

    /// Solarized Dark: classic Solarized dark palette
    pub fn solarized_dark() -> Self {
        Self {
            bg: Color::Rgb(0, 43, 54),
            bg_secondary: Color::Rgb(7, 54, 66),
            bg_panel: Color::Rgb(15, 65, 78),
            fg: Color::Rgb(131, 148, 150),
            fg_secondary: Color::Rgb(108, 113, 196),
            fg_dimmed: Color::Rgb(88, 110, 117),
            accent: Color::Rgb(38, 139, 210),
            accent_secondary: Color::Rgb(181, 137, 0),
            info: Color::Rgb(38, 139, 210),
            highlight: Color::Rgb(238, 232, 213),
            success: Color::Rgb(133, 153, 0),
            warning: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            gold: Color::Rgb(181, 137, 0),
            border: Color::Rgb(7, 54, 66),
        }
    }

    /// Gruvbox Dark: warm retro palette
    pub fn gruvbox_dark() -> Self {
        Self {
            bg: Color::Rgb(40, 40, 40),
            bg_secondary: Color::Rgb(50, 48, 45),
            bg_panel: Color::Rgb(60, 56, 54),
            fg: Color::Rgb(235, 219, 178),
            fg_secondary: Color::Rgb(213, 196, 154),
            fg_dimmed: Color::Rgb(146, 131, 116),
            accent: Color::Rgb(251, 191, 36),
            accent_secondary: Color::Rgb(184, 187, 38),
            info: Color::Rgb(69, 133, 136),
            highlight: Color::Rgb(235, 219, 178),
            success: Color::Rgb(152, 195, 121),
            warning: Color::Rgb(250, 189, 47),
            error: Color::Rgb(251, 73, 52),
            gold: Color::Rgb(214, 157, 0),
            border: Color::Rgb(80, 73, 69),
        }
    }

    /// High Contrast Accessibility: maximum readability
    pub fn high_contrast() -> Self {
        Self {
            bg: Color::Black,
            bg_secondary: Color::Rgb(10, 10, 10),
            bg_panel: Color::Rgb(20, 20, 20),
            fg: Color::Rgb(255, 255, 255),
            fg_secondary: Color::Rgb(220, 220, 220),
            fg_dimmed: Color::Rgb(170, 170, 170),
            accent: Color::Rgb(0, 255, 255),
            accent_secondary: Color::Rgb(100, 200, 255),
            info: Color::Rgb(80, 80, 255),
            highlight: Color::Rgb(255, 255, 0),
            success: Color::Rgb(0, 255, 100),
            warning: Color::Rgb(255, 255, 0),
            error: Color::Rgb(255, 50, 50),
            gold: Color::Rgb(255, 255, 100),
            border: Color::Rgb(150, 150, 150),
        }
    }

    /// 16-color fallback: works on any terminal
    pub fn fallback_16() -> Self {
        Self {
            bg: Color::Black,
            bg_secondary: Color::Black,
            bg_panel: Color::Rgb(25, 25, 25),
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
            border: Color::DarkGray,
        }
    }
}

use std::sync::OnceLock;

pub const MARK: &str = "◈";
pub const ACTIVE: &str = "●";
pub const INACTIVE: &str = "○";
pub const ACCENT: &str = "◆";
pub const ARROW: &str = "→";
pub const CHECK: &str = "✓";
pub const CROSS: &str = "✗";
pub const RUNNING: &str = "●";
pub const PENDING: &str = "○";
pub const BLOCKED: &str = "[-]";
pub const THINKING: &str = "▶";
pub const CURSOR: &str = "▌";

pub const DIVIDER: &str = "─";
pub const DIVIDER_STEPPED: &str = "─ ── ────── ── ─ ────── ── ─";
pub const SECTION_LINE: &str = "═══";

pub const BORDER_TOP_LEFT: &str = "╭";
pub const BORDER_TOP_RIGHT: &str = "╮";
pub const BORDER_BOTTOM_LEFT: &str = "╰";
pub const BORDER_BOTTOM_RIGHT: &str = "╯";
pub const BORDER_HORIZONTAL: &str = "─";
pub const BORDER_VERTICAL: &str = "│";

const EYE_OF_HORUS_RICH: &str = "𓂀";
const ANKH_RICH: &str = "𓋹";

const EYE_OF_HORUS_PLAIN: &str = "<*>";
const ANKH_PLAIN: &str = "+";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphTier {
    Rich,
    Safe,
    Plain,
}

static GLYPH_TIER: OnceLock<GlyphTier> = OnceLock::new();

pub fn detect_tier() -> GlyphTier {
    *GLYPH_TIER.get_or_init(detect_tier_inner)
}

fn detect_tier_inner() -> GlyphTier {
    let term = std::env::var("TERM").unwrap_or_default().to_lowercase();
    if term == "dumb" || term == "unknown" || term.starts_with("vt100") || term == "linux" {
        return GlyphTier::Plain;
    }

    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        let tp_lower = tp.to_lowercase();
        let capable = [
            "iterm",
            "wezterm",
            "kitty",
            "alacritty",
            "ghostty",
            "rio",
            "warp",
            "hyper",
            "tabby",
            "vscode",
            "jetbrains",
            "mintty",
            "konsole",
            "gnome-terminal",
            "kgx",
        ];
        if capable.iter().any(|c| tp_lower.contains(c)) {
            return GlyphTier::Rich;
        }
    }

    if term.contains("256color") || term.contains("truecolor") || term.contains("direct") {
        return GlyphTier::Rich;
    }

    let lang = std::env::var("LANG").unwrap_or_default().to_lowercase();
    let lc_ctype = std::env::var("LC_CTYPE").unwrap_or_default().to_lowercase();
    let has_utf8 = lang.contains("utf-8")
        || lang.contains("utf8")
        || lc_ctype.contains("utf-8")
        || lc_ctype.contains("utf8");

    if has_utf8 && !term.contains("screen") && !term.starts_with("vt") {
        return GlyphTier::Rich;
    }

    if std::env::var("COLORFGBG").is_ok() {
        return GlyphTier::Rich;
    }

    GlyphTier::Safe
}

pub fn set_tier(tier: GlyphTier) {
    let _ = GLYPH_TIER.set(tier);
}

pub fn logo_glyph() -> &'static str {
    match detect_tier() {
        GlyphTier::Rich => EYE_OF_HORUS_RICH,
        GlyphTier::Safe => MARK,
        GlyphTier::Plain => EYE_OF_HORUS_PLAIN,
    }
}

pub fn ornament() -> &'static str {
    match detect_tier() {
        GlyphTier::Rich | GlyphTier::Safe => ACCENT,
        GlyphTier::Plain => "-",
    }
}

pub fn ankh() -> &'static str {
    match detect_tier() {
        GlyphTier::Rich => ANKH_RICH,
        GlyphTier::Safe => ACCENT,
        GlyphTier::Plain => ANKH_PLAIN,
    }
}

pub fn status_glyph(running: bool) -> &'static str {
    if running {
        ACTIVE
    } else {
        INACTIVE
    }
}

pub fn task_status_glyph(status: &fever_core::TaskStatus) -> &'static str {
    match status {
        fever_core::TaskStatus::Queued => PENDING,
        fever_core::TaskStatus::Running => RUNNING,
        fever_core::TaskStatus::Completed => CHECK,
        fever_core::TaskStatus::Failed => CROSS,
        fever_core::TaskStatus::Blocked => BLOCKED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_tier_returns_known_variant() {
        let tier = detect_tier();
        matches!(tier, GlyphTier::Rich | GlyphTier::Safe | GlyphTier::Plain);
    }

    #[test]
    fn logo_glyph_returns_non_empty() {
        assert!(!logo_glyph().is_empty());
    }

    #[test]
    fn ankh_returns_non_empty() {
        assert!(!ankh().is_empty());
    }

    #[test]
    fn set_tier_returns_known_value() {
        let tier = detect_tier();
        matches!(tier, GlyphTier::Rich | GlyphTier::Safe | GlyphTier::Plain);
    }

    #[test]
    fn ornament_returns_non_empty() {
        assert!(!ornament().is_empty());
    }
}

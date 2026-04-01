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

pub const DIVIDER: &str = "─";
pub const DIVIDER_STEPPED: &str = "─ ── ────── ── ─ ────── ── ─";
pub const SECTION_LINE: &str = "═══";

pub const EYE_OF_HORUS: &str = "𓂀";
pub const ANKH: &str = "𓋹";

pub const BORDER_TOP_LEFT: &str = "╭";
pub const BORDER_TOP_RIGHT: &str = "╮";
pub const BORDER_BOTTOM_LEFT: &str = "╰";
pub const BORDER_BOTTOM_RIGHT: &str = "╯";
pub const BORDER_HORIZONTAL: &str = "─";
pub const BORDER_VERTICAL: &str = "│";

pub fn status_glyph(running: bool) -> &'static str {
    if running { ACTIVE } else { INACTIVE }
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

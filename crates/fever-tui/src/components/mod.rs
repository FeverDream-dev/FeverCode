pub mod input_bar;
pub mod logo;
pub mod message;
pub mod progress;
pub mod status_bar;
pub mod tool_card;

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::theme::Theme;

pub trait Component {
    fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme);
}

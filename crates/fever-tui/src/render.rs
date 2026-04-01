use crate::app::AppState;
use crate::components::status_bar::StatusBar;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

pub fn render_frame(f: &mut Frame, state: &mut AppState) {
    let size = f.area();

    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(size);

    f.render_widget(
        ratatui::widgets::Block::default()
            .style(ratatui::style::Style::default().bg(state.theme.bg())),
        size,
    );

    state.render_screen(f, chunks[0]);

    let mut sb = StatusBar::new();
    sb.provider = state.provider_name.clone();
    sb.model = state.model_name.clone();
    sb.workspace = state.workspace.clone();
    sb.streaming = state.streaming;
    sb.render(f, chunks[1], &state.theme);
}

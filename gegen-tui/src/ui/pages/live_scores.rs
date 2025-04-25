use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use throbber_widgets_tui::ThrobberState;

use crate::{State, state::LiveData};

fn calculate_loading_layout(area: Rect) -> [Rect; 2] {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    main_layout.areas(area)
}

fn calculate_loaded_layout(area: Rect) -> (Rect, Vec<Vec<Rect>>) {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    let block_layout = Layout::vertical([Constraint::Max(4); 9]);
    let [title_area, main_area] = main_layout.areas(area);
    let main_areas = block_layout
        .split(main_area)
        .iter()
        .map(|&area| {
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area)
                .to_vec()
        })
        .collect();
    (title_area, main_areas)
}

fn render_title(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Block example. Press q to quit")
            .dark_gray()
            .alignment(Alignment::Center),
        area,
    );
}

fn render_loading(frame: &mut Frame, area: Rect, throbber_state: &mut ThrobberState) {
    let full = throbber_widgets_tui::Throbber::default()
        .label("Running...")
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
        .throbber_style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Red)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::ARROW)
        .use_type(throbber_widgets_tui::WhichUse::Spin);
    frame.render_stateful_widget(full, area, throbber_state);
}

fn placeholder_paragraph() -> Paragraph<'static> {
    let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
    Paragraph::new(text.dark_gray()).wrap(Wrap { trim: true })
}

fn render_borders(paragraph: &Paragraph, frame: &mut Frame, area: Rect) {
    let block = Block::new().borders(Borders::all());
    frame.render_widget(paragraph.clone().block(block), area);
}

pub(crate) fn draw(frame: &mut Frame, app_state: &mut State) {
    tracing::info!("get read lock");
    let data = app_state.data.read().unwrap();
    match data.as_ref() {
        Some(live_data) => {
            let (title_area, layout) = calculate_loaded_layout(frame.area());
            render_title(frame, title_area);

            let paragraph = placeholder_paragraph();

            render_borders(&paragraph, frame, layout[0][0]);
            render_borders(&paragraph, frame, layout[0][1]);
        }
        None => {
            let [title_area, layout] = calculate_loading_layout(frame.area());

            render_title(frame, title_area);
            render_loading(
                frame,
                layout,
                &mut app_state.page_states.live_scores.throbber_state,
            );
        }
    }
    drop(data);
    tracing::info!("drop read lock");
}

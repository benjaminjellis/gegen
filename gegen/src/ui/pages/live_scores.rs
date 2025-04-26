use chrono::NaiveDate;
use itertools::Itertools;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use throbber_widgets_tui::ThrobberState;

use crate::State;

fn calculate_loading_layout(area: Rect) -> [Rect; 2] {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    main_layout.areas(area)
}

fn calculate_loaded_layout(area: Rect) -> (Rect, Vec<Vec<Rect>>) {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    let block_layout = Layout::vertical([Constraint::Percentage(25); 4]);
    let [title_area, main_area] = main_layout.areas(area);
    let main_areas = block_layout
        .split(main_area)
        .iter()
        .map(|&area| {
            Layout::horizontal([Constraint::Percentage(100)])
                .split(area)
                .to_vec()
        })
        .collect();
    (title_area, main_areas)
}

fn render_title(frame: &mut Frame, area: Rect, date: &NaiveDate, todays_date: &NaiveDate) {
    let title = if date == todays_date {
        format!("Today ({date})")
    } else {
        format!("{date}")
    };

    frame.render_widget(
        Paragraph::new(title)
            .light_green()
            .alignment(Alignment::Center),
        area,
    );
}

// TODO: make throbber render in centre of page
fn render_loading(frame: &mut Frame, area: Rect, throbber_state: &mut ThrobberState) {
    let full = throbber_widgets_tui::Throbber::default()
        .label("loading...")
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

fn render_borders(paragraph: &Paragraph, frame: &mut Frame, area: Rect, title: String) {
    let block = Block::new()
        .borders(Borders::all())
        .title(title)
        .title_style(Style::new().blue());
    frame.render_widget(paragraph.clone().block(block), area);
}

pub(crate) fn draw(frame: &mut Frame, app_state: &mut State, date: &NaiveDate) {
    match app_state.data.get(date) {
        Some(data) => {
            let mut data_grouped = Vec::new();
            for (key, chunk) in &data.matches.iter().chunk_by(|m| &m.comp.name) {
                data_grouped.push((key, chunk.collect::<Vec<_>>()));
            }

            let (title_area, layout) = calculate_loaded_layout(frame.area());
            render_title(frame, title_area, date, &app_state.today);

            for (idx, (key, _)) in data_grouped.into_iter().take(4).enumerate() {
                let paragraph = Paragraph::new("test".dark_gray()).wrap(Wrap { trim: true });
                render_borders(&paragraph, frame, layout[idx][0], key.clone());
            }
        }
        None => {
            app_state.fetch_data_for_date(*date);
            let [title_area, layout] = calculate_loading_layout(frame.area());

            render_title(frame, title_area, date, &app_state.today);
            render_loading(
                frame,
                layout,
                &mut app_state.page_states.live_scores.throbber_state,
            );
        }
    }
    tracing::info!("drop read lock");
}

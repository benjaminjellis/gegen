use chrono::{Datelike, NaiveDate};
use itertools::Itertools;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Tabs, Widget},
};
use throbber_widgets_tui::ThrobberState;

use crate::State;

const FIXTURES_PER_ROW: usize = 5;

fn calculate_loading_layout(area: Rect) -> [Rect; 2] {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    main_layout.areas(area)
}

fn render_title(frame: &mut Frame, area: Rect, date: &NaiveDate, todays_date: &NaiveDate) {
    let weekday = date.weekday();
    let title = if date == todays_date {
        format!("Today ({weekday} - {date})")
    } else {
        format!("{weekday} - {date}")
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

pub(crate) fn draw(frame: &mut Frame, app_state: &mut State, date: &NaiveDate) {
    match app_state.get_grouped_data() {
        Some(data_grouped) => {
            let vertical = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ]);
            let [header_area, inner_area, footer_area] = vertical.areas(frame.area());

            let horizontal = Layout::vertical([Constraint::Min(1), Constraint::Percentage(100)]);
            let [tabs_area, content_area] = horizontal.areas(inner_area);

            let titles = data_grouped.iter().map(|(key, _)| {
                let mut tab_title = key.to_string();
                tab_title.truncate(5);
                Line::from(tab_title)
            });

            render_title(frame, header_area, date, &app_state.today);

            // tabs
            let highlight_style = Style::new().bg(Color::Green).fg(Color::Magenta);
            Tabs::new(titles)
                .highlight_style(highlight_style)
                .select(Some(*app_state.selected_tab()))
                .padding("", "")
                .render(tabs_area, frame.buffer_mut());

            let block = Block::bordered()
                .border_set(symbols::border::PROPORTIONAL_TALL)
                // .padding(Padding::horizontal(1))
                .border_style(Color::Green);
            Paragraph::new("Welcome to the Ratatui tabs example!")
                .block(block)
                .render(content_area, frame.buffer_mut());

            let st = app_state.page_states.live_scores.selected_tab;

            let tab_title = match data_grouped.get(st) {
                Some((key, _)) => key.clone(),
                None => "".into(),
            };

            Paragraph::new(tab_title)
                .style(Style::new().magenta())
                .render(footer_area, frame.buffer_mut());
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

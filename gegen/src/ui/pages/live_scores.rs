use chrono::{Datelike, NaiveDate};
use gegen_data::types::{Match, Team};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Text},
    widgets::{Block, Cell, Paragraph, Row, Table, Tabs, Widget},
};
use throbber_widgets_tui::ThrobberState;

use crate::State;

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
            .bold()
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
            let highlight_style = Style::new().bg(Color::Green).fg(Color::Magenta).bold();
            Tabs::new(titles)
                .highlight_style(highlight_style)
                .select(Some(*app_state.selected_tab()))
                .padding("", "")
                .render(tabs_area, frame.buffer_mut());

            let block = Block::bordered()
                .border_set(symbols::border::PROPORTIONAL_TALL)
                // .padding(Padding::horizontal(1))
                .border_style(Color::Green);

            let st = app_state.page_states.live_scores.selected_tab;

            let tab_title = match data_grouped.get(st) {
                Some((key, _)) => key.clone(),
                None => "".into(),
            };

            Paragraph::new(tab_title)
                .style(Style::new().magenta())
                .render(footer_area, frame.buffer_mut());

            let Some(fixtures) = data_grouped.get(*app_state.selected_tab()) else {
                return;
            };

            let rows = fixtures
                .1
                .iter()
                .enumerate()
                .map(|(idx, fixture)| build_row(idx, fixture))
                .collect::<Vec<_>>();

            let table = Table::new(
                rows,
                [
                    Constraint::Min(0),
                    Constraint::Min(10),
                    Constraint::Percentage(50),
                    Constraint::Min(10),
                    Constraint::Percentage(50),
                ],
            )
            .block(block);

            frame.render_stateful_widget(
                table,
                content_area,
                &mut app_state.page_states.live_scores.table_state,
            );
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

fn build_row(idx: usize, fixture: &Match) -> Row {
    let (row_color, text_color) = match idx % 2 {
        0 => (Color::Black, Color::White),
        _ => (Color::Gray, Color::Black),
    };

    let (state_text, state_style, center_text, center_style) = match fixture.period {
        // first half and second half
        1 | 2 => {
            let scores = fixture.score.clone().unwrap_or_default();

            let current_score = scores.get(&gegen_data::types::ScoreKey::Total).expect(
                "period is 1 or 2 (first of second half) but no total scores were provided",
            );
            let time = &fixture.time.unwrap_or(0);

            (
                format!("{time}'"),
                Style::new().red().bold().italic(),
                format!("{} - {}", current_score.home, current_score.away),
                Style::new().red().bold(),
            )
        }
        // yet to start
        16 => {
            let start_time = &fixture.date.naive_local().time().format("%H:%M");

            (
                format!("{start_time}"),
                Style::new().fg(text_color).bold(),
                "v".to_string(),
                Style::new().fg(text_color).bold(),
            )
        }
        // half time
        10 => {
            let scores = fixture.score.clone().unwrap_or_default();

            let current_score = scores
                .get(&gegen_data::types::ScoreKey::Ht)
                .expect("period is 10 (half time) but no half time scores were provided");

            (
                "ht".to_string(),
                Style::new().red().bold(),
                format!("{} - {}", current_score.home, current_score.away),
                Style::new().red().bold(),
            )
        }
        // full time
        14 => {
            let scores = fixture.score.clone().unwrap_or_default();

            let current_score = scores
                .get(&gegen_data::types::ScoreKey::Ft)
                .expect("period was 14 (full time) but no full time score was provided");

            (
                "ft".to_string(),
                Style::new().fg(text_color).bold(),
                format!("{} - {}", current_score.home, current_score.away),
                Style::new().fg(text_color).bold(),
            )
        }
        _ => {
            tracing::error!("got handled period for fixtrue: {fixture:?}");
            (
                "?".to_string(),
                Style::new().fg(text_color).bold(),
                "?".to_string(),
                Style::new().red().bold(),
            )
        }
    };

    Row::new(vec![
        Cell::new(""),
        Cell::new(
            Text::from(state_text)
                .alignment(Alignment::Left)
                .style(state_style),
        ),
        format_team_name(&fixture.home, Alignment::Right, text_color),
        Cell::new(
            Text::from(center_text)
                .alignment(Alignment::Center)
                .style(center_style),
        ),
        format_team_name(&fixture.away, Alignment::Left, text_color),
    ])
    .style(Style::new().bg(row_color))
}

fn format_team_name(team: &Team, alligment: Alignment, text_color: Color) -> Cell {
    Cell::new(
        Text::from(team.name.clone().unwrap_or("tbc".into()))
            .alignment(alligment)
            .style(text_color),
    )
}

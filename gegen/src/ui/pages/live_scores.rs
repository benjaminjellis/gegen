use chrono::{Local, NaiveDate};
use gegen_data::types::{Match, ScoreKey, Team};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::{Line, Text},
    widgets::{Block, Cell, Row, Table, Tabs, Widget},
};

use crate::{PageRenderStates, State};

use super::shared::{render_loading, render_title};

fn calculate_loading_layout(area: Rect) -> [Rect; 2] {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    main_layout.areas(area)
}

pub(crate) fn draw(
    frame: &mut Frame,
    app_state: &State,
    render_state: &mut PageRenderStates,
    date: &NaiveDate,
) {
    match app_state.get_grouped_data() {
        Some(data_grouped) => {
            let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
            let [header_area, inner_area] = vertical.areas(frame.area());

            let horizontal = Layout::vertical([Constraint::Min(1), Constraint::Percentage(100)]);
            let [tabs_area, content_area] = horizontal.areas(inner_area);

            let selected_tab = render_state.live_scores.selected_tab;

            let tab_title = match data_grouped.get(selected_tab) {
                Some((key, _)) => key.clone(),
                None => "".into(),
            };

            let titles = data_grouped.iter().map(|(key, _)| {
                let mut tab_title = key.to_string();
                tab_title.truncate(5);
                Line::from(tab_title)
            });

            render_title(frame, header_area, date, &app_state.today, tab_title);

            // tabs
            let highlight_style = Style::new().bg(Color::Green).fg(Color::Magenta).bold();
            Tabs::new(titles)
                .highlight_style(highlight_style)
                .select(Some(selected_tab))
                .padding("", "")
                .render(tabs_area, frame.buffer_mut());

            let block = Block::bordered()
                .border_set(symbols::border::DOUBLE)
                .border_style(Color::Green);

            let Some(fixtures) = data_grouped.get(selected_tab) else {
                return;
            };

            let rows = fixtures
                .1
                .iter()
                .enumerate()
                .map(|(idx, fixture)| build_row(idx, fixture))
                .collect::<Vec<_>>();

            let selected_row_style = Style::default().bg(Color::White).fg(Color::LightMagenta);

            let table = Table::new(
                rows,
                [
                    Constraint::Min(0),
                    Constraint::Min(10),
                    Constraint::Percentage(50),
                    Constraint::Min(20),
                    Constraint::Percentage(50),
                ],
            )
            .row_highlight_style(selected_row_style)
            .block(block);

            frame.render_stateful_widget(
                table,
                content_area,
                &mut render_state.live_scores.table_state,
            );
        }
        None => {
            app_state.fetch_data_for_date(*date);
            let [title_area, layout] = calculate_loading_layout(frame.area());

            render_title(frame, title_area, date, &app_state.today, "".to_string());
            render_loading(frame, layout, &mut render_state.live_scores.throbber_state);
        }
    }
}

fn build_row<'a>(idx: usize, fixture: &'a Match) -> Row<'a> {
    let (row_color, text_color) = match idx % 2 {
        0 => (Color::Black, Color::White),
        _ => (Color::Green, Color::Black),
    };

    let aggregate_score = fixture.try_get_score(&ScoreKey::Aggregate);

    let (state_text, state_style, center_text, center_style) = match fixture.period {
        // first half and second half
        1 | 2 => {
            let unconfimed_score = fixture.try_get_score(&ScoreKey::TotalUnconfirmed);

            let score = if let Some(score) = unconfimed_score {
                if let Some(aggregate_score) = aggregate_score {
                    format!(
                        "{}[{}] - {}[{}] (*)",
                        score.home, aggregate_score.home, score.away, aggregate_score.away
                    )
                } else {
                    format!("{} - {} (*)", score.home, score.away)
                }
            } else {
                let current_score = fixture.try_get_score(&ScoreKey::Total).expect(
                    "period is 1 or 2 (first of second half) but no total score was provided",
                );
                if let Some(aggregate_score) = aggregate_score {
                    format!(
                        "{} [{}] - {} [{}] (*)",
                        current_score.home,
                        aggregate_score.home,
                        current_score.away,
                        aggregate_score.away
                    )
                } else {
                    format!("{} - {}", current_score.home, current_score.away)
                }
            };

            let time = &fixture.time.unwrap_or(0);
            (
                format!("{time}'"),
                Style::new().red().bold().italic(),
                score,
                Style::new().red().bold().italic(),
            )
        }
        // yet to start
        16 => {
            let start_time = &fixture.date.with_timezone(&Local).time().format("%H:%M");

            let center_text = if let Some(aggregate) = aggregate_score {
                format!("[{}] v [{}]", aggregate.home, aggregate.away)
            } else {
                "v".to_string()
            };
            (
                format!("{start_time}"),
                Style::new().fg(text_color).bold(),
                center_text,
                Style::new().fg(text_color).bold(),
            )
        }
        // half time
        10 => {
            let center_text = if let Some(ht_score) = fixture.try_get_score(&ScoreKey::Ht) {
                if let Some(aggregate_score) = fixture.try_get_score(&ScoreKey::Aggregate) {
                    format!(
                        "{} [{}] - {} [{}]",
                        ht_score.home, aggregate_score.home, ht_score.away, aggregate_score.away
                    )
                } else {
                    format!("{} - {}", ht_score.home, ht_score.away)
                }
            } else {
                "???".to_string()
            };

            (
                "ht".to_string(),
                Style::new().red().bold(),
                center_text,
                Style::new().red().bold(),
            )
        }
        // full time
        14 => {
            let current_score = fixture
                .try_get_score(&ScoreKey::Total)
                .expect("period was 14 (full time) but no full time score was provided");

            let score = if let Some(aggregate_score) = aggregate_score {
                format!(
                    "{} [{}] - {} [{}]",
                    current_score.home,
                    aggregate_score.home,
                    current_score.away,
                    aggregate_score.away
                )
            } else {
                format!("{} - {}", current_score.home, current_score.away)
            };

            (
                "ft".to_string(),
                Style::new().fg(text_color).bold(),
                score,
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

fn format_team_name<'a>(team: &'a Team, alligment: Alignment, text_color: Color) -> Cell<'a> {
    Cell::new(
        Text::from(team.name.clone().unwrap_or("tbc".into()))
            .alignment(alligment)
            .style(text_color),
    )
}

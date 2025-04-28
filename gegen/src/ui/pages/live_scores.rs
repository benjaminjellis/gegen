use chrono::NaiveDate;
use itertools::Itertools;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::collections::VecDeque;
use throbber_widgets_tui::ThrobberState;

use crate::State;

const LEAGUES_PER_SCREEN: usize = 4;
const FIXTURES_PER_ROW: usize = 5;

fn calculate_loading_layout(area: Rect) -> [Rect; 2] {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
    main_layout.areas(area)
}

fn calculate_loaded_layout(
    area: Rect,
    fixtures_per_league: Vec<usize>,
) -> (Rect, Vec<Rect>, Vec<Vec<Rect>>) {
    let main_layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);

    let block_layout = Layout::vertical([Constraint::Ratio(1, 4); LEAGUES_PER_SCREEN]);
    let [title_area, main_area] = main_layout.areas(area);
    // this vec should be of length == LEAGUES_PER_SCREEN
    let main_areas = block_layout.split(main_area).to_vec();

    let mut fixture_blocks: Vec<Vec<Rect>> = vec![];

    for (area, no_fixtures) in main_areas.iter().zip(fixtures_per_league) {
        let no_rows = no_fixtures.div_ceil(FIXTURES_PER_ROW);

        let mut a: VecDeque<_> =
            vec![Constraint::Percentage((80 / no_rows) as u16); no_rows].into();
        a.push_front(Constraint::Fill(1));
        a.push_back(Constraint::Fill(1));

        let rows = Layout::vertical(a);
        let rows = rows.split(*area).to_vec();
        let mut fbs = vec![];
        for row in &rows[1..rows.len() - 1] {
            let mut l: VecDeque<_> = vec![Constraint::Percentage(20); FIXTURES_PER_ROW].into();
            l.push_front(Constraint::Percentage(2));
            l.push_back(Constraint::Percentage(2));

            let layout = Layout::horizontal(l);

            let fixtures = layout.split(*row).to_vec();

            fbs.extend(&fixtures[1..fixtures.len() - 1]);
        }
        fixture_blocks.push(fbs);
    }
    (title_area, main_areas, fixture_blocks)
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

fn render_borders(frame: &mut Frame, area: Rect, title: String) {
    let block = Block::new()
        .borders(Borders::all())
        .title(title)
        .title_style(Style::new().blue());
    frame.render_widget(block, area);
}

pub(crate) fn draw(frame: &mut Frame, app_state: &mut State, date: &NaiveDate) {
    match app_state.data.get(date) {
        Some(data) => {
            let mut data_grouped = Vec::new();
            for (key, chunk) in &data
                .matches
                .iter()
                .chunk_by(|m| format!("{} - {}", m.comp.country.full_name, m.comp.name,))
            {
                data_grouped.push((key, chunk.collect::<Vec<_>>()));
            }

            // TODO: handle when vertical_scroll is larger than the number of leagues by >1
            let offset = app_state
                .page_states
                .live_scores
                .vertical_scroll
                .min(data_grouped.len() - LEAGUES_PER_SCREEN);

            let slice = &data_grouped[offset..offset + LEAGUES_PER_SCREEN];

            let fixtures_per_league = slice.iter().map(|(_, g)| g.len()).collect();

            let (title_area, layout, fixtures_blocks) =
                calculate_loaded_layout(frame.area(), fixtures_per_league);
            render_title(frame, title_area, date, &app_state.today);

            for (idx, (key, _)) in slice.iter().enumerate() {
                render_borders(frame, layout[idx], key.to_string());
                for a in &fixtures_blocks[idx] {
                    let block = Block::new().borders(Borders::all());
                    frame.render_widget(block, *a);
                }
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

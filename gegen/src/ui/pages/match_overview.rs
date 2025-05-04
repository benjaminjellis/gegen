use chrono::NaiveDate;
use gegen_data::types::Match;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::Paragraph,
};

use crate::{PageRenderStates, State};

use super::shared::{render_loading, render_title};

pub(crate) fn draw(
    frame: &mut Frame,
    app_state: &State,
    render_state: &mut PageRenderStates,
    date: &NaiveDate,
    match_id: &str,
    competition_name: &String,
) {
    let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);

    let [header_area, inner_area] = vertical.areas(frame.area());
    render_title(
        frame,
        header_area,
        date,
        &app_state.today,
        competition_name.clone(),
    );

    match app_state.get_grouped_data() {
        Some(data_grouped) => {
            let Some(competition_data) = data_grouped
                .iter()
                .find(|(comp_name, _)| comp_name == competition_name)
            else {
                render_loading(
                    frame,
                    inner_area,
                    &mut render_state.match_overview.throbber_state,
                );
                return;
            };

            let Some(match_data) = competition_data.1.iter().find(|m| m.id == match_id) else {
                render_loading(
                    frame,
                    inner_area,
                    &mut render_state.match_overview.throbber_state,
                );
                return;
            };

            let layout = Layout::vertical([
                Constraint::Percentage(2),
                Constraint::Percentage(8),
                Constraint::Percentage(90),
            ]);
            let [time_area, overview, events] = layout.areas(inner_area);

            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Min(10),
                Constraint::Percentage(50),
            ]);
            let [home_team_area, score_area, away_team_area] = layout.areas(overview);
            draw_overview(
                frame,
                match_data,
                time_area,
                home_team_area,
                score_area,
                away_team_area,
            );
        }
        None => {
            render_loading(
                frame,
                inner_area,
                &mut render_state.match_overview.throbber_state,
            );
        }
    }
}

fn draw_overview(
    frame: &mut Frame,
    match_data: &Match,
    time_area: Rect,
    home_team_area: Rect,
    score_area: Rect,
    away_team_area: Rect,
) {
    let home_team_para = Paragraph::new(match_data.home.clone().name.unwrap_or("tbc".into()))
        .bold()
        .centered();
    let away_team_para = Paragraph::new(match_data.away.clone().name.unwrap_or("tbc".into()))
        .bold()
        .centered();

    frame.render_widget(home_team_para, home_team_area);
    frame.render_widget(away_team_para, away_team_area);

    match match_data.period {
        // first or second half
        1 | 2 => {
            let match_time = match_data.time.unwrap_or_default();
            let match_time_string = format!("{match_time}'");
            let match_time_para = Paragraph::new(match_time_string)
                .centered()
                .red()
                .bold()
                .italic();
            frame.render_widget(match_time_para, time_area);
        }
        // penalties
        5 => {
            let p = Paragraph::new("penalties").centered().red().bold().italic();
            frame.render_widget(p, time_area);
        }
        // half time
        10 => {
            let p = Paragraph::new("ht").centered().bold().red();
            frame.render_widget(p, time_area);
        }
        // full time
        14 => {
            let p = Paragraph::new("ft").centered().bold();
            frame.render_widget(p, time_area);
        }
        // yet to start
        16 => {
            let start_time = &match_data.date.naive_local().time().format("%H:%M");
            let p = Paragraph::new(format!("{start_time}")).centered().bold();
            frame.render_widget(p, time_area);
        }
        _ => {}
    }
}

fn draw_events(match_data: &Match) {
    let Some(events) = &match_data.events else {
        return;
    };
    for event in events {}
}

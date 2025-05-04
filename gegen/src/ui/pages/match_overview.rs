use chrono::NaiveDate;
use gegen_data::types::{
    Card, CardEvent, Event, GoalEvent, GoalType, Match, PenaltyEvent, ScoreKey, SubEvent, VAREvent,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    symbols,
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table},
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

    let block = Block::bordered()
        .border_set(symbols::border::DOUBLE)
        .border_style(Color::Green);

    frame.render_widget(block, inner_area);

    let layout = Layout::vertical([
        Constraint::Min(1),
        Constraint::Percentage(100),
        Constraint::Min(1),
    ]);

    let [_, inner_area, _] = layout.areas(inner_area);

    let layout = Layout::horizontal([
        Constraint::Min(1),
        Constraint::Percentage(100),
        Constraint::Min(1),
    ]);

    let [_, inner_area, _] = layout.areas(inner_area);

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
            let [time_area, overview_area, events_area] = layout.areas(inner_area);

            let layout = Layout::horizontal([
                Constraint::Percentage(50),
                Constraint::Min(10),
                Constraint::Percentage(50),
            ]);
            let [home_team_area, score_area, away_team_area] = layout.areas(overview_area);
            draw_overview(
                frame,
                match_data,
                time_area,
                home_team_area,
                score_area,
                away_team_area,
            );
            draw_events(frame, match_data, events_area, render_state);
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
            let scores = match_data.score.clone().unwrap_or_default();

            let unconfimed_score = scores.get(&gegen_data::types::ScoreKey::TotalUnconfirmed);
            let score = if let Some(score) = unconfimed_score {
                format!("{} - {} (*)", score.home, score.away)
            } else {
                let current_score = scores.get(&gegen_data::types::ScoreKey::Total).expect(
                    "period is 1 or 2 (first of second half) but no total score was provided",
                );
                format!("{} - {}", current_score.home, current_score.away)
            };

            let match_time = match_data.time.unwrap_or_default();
            let match_time_string = format!("{match_time}'");
            let match_time_para = Paragraph::new(match_time_string)
                .centered()
                .red()
                .bold()
                .italic();

            let score_para = Paragraph::new(score).red().italic().bold().centered();
            frame.render_widget(match_time_para, time_area);
            frame.render_widget(score_para, score_area);
        }
        // penalties
        5 => {
            let p = Paragraph::new("penalties").centered().red().bold().italic();
            frame.render_widget(p, time_area);
        }
        // half time
        10 => {
            let scores = match_data.score.clone().unwrap_or_default();
            let ht_score = scores.get(&ScoreKey::Ht).expect("No half time score");
            let ht_score = format!("{} - {}", ht_score.home, ht_score.away);

            let score_para = Paragraph::new(ht_score).bold().centered();

            let p = Paragraph::new("ht").centered().bold().red();
            frame.render_widget(p, time_area);
            frame.render_widget(score_para, score_area);
        }
        // full time
        14 => {
            let scores = match_data.score.clone().unwrap_or_default();
            let ft_score = scores.get(&ScoreKey::Ft).expect("No full time score");
            let ft_score = format!("{} - {}", ft_score.home, ft_score.away);

            let score_para = Paragraph::new(ft_score).bold().centered();
            let p = Paragraph::new("ft").centered().bold();
            frame.render_widget(p, time_area);
            frame.render_widget(score_para, score_area);
        }
        // yet to start
        16 => {
            let start_time = &match_data.date.naive_local().time().format("%H:%M");
            let p = Paragraph::new(format!("{start_time}")).centered().bold();
            frame.render_widget(p, time_area);
            let score_para = Paragraph::new("vs").bold().centered();
            frame.render_widget(score_para, score_area);
        }
        _ => {}
    }
}

fn draw_events(
    frame: &mut Frame,
    match_data: &Match,
    events_area: Rect,
    render_state: &mut PageRenderStates,
) {
    let Some(events) = &match_data.events else {
        return;
    };

    let event_rows = events
        .iter()
        .map(|event| render_event(event, match_data.home.id.as_ref()))
        .collect::<Vec<_>>();

    let table = Table::new(
        event_rows,
        [
            Constraint::Percentage(50),
            Constraint::Min(2),
            Constraint::Min(5),
            Constraint::Min(2),
            Constraint::Percentage(50),
        ],
    );
    frame.render_stateful_widget(
        table,
        events_area,
        &mut render_state.live_scores.table_state,
    );
}

enum EventSide {
    Home,
    Away,
}

fn render_event(event: &Event, home_team_id: Option<&String>) -> Row<'static> {
    let (emoji, text, event_side) = match event {
        Event::Sub(sub_event) => build_sub_event(sub_event, home_team_id),
        // ("🔄", "sub".into(), EventSide::Home),
        Event::Goal(goal_event) => build_goal_event(goal_event, home_team_id),
        Event::Card(card_event) => build_card_event(card_event, home_team_id),
        Event::Var(var_event) => build_var_event(var_event, home_team_id),
        Event::Pen(penalty_event) => build_penalty_event(penalty_event, home_team_id),
    };

    let time = Cell::new(Text::from(event.get_time_str().clone()));
    let emoji = Cell::new(emoji);

    let v = match event_side {
        EventSide::Home => [
            Cell::new(text.right_aligned()),
            emoji,
            time,
            Cell::new(""),
            Cell::new(""),
        ],
        EventSide::Away => [Cell::new(""), Cell::new(""), time, emoji, Cell::new(text)],
    };

    Row::new(v)
}

fn build_penalty_event(
    penalty_event: &PenaltyEvent,
    home_team_id: Option<&String>,
) -> (&'static str, Text<'static>, EventSide) {
    let event_side = if Some(&penalty_event.team_id) == home_team_id {
        EventSide::Home
    } else {
        EventSide::Away
    };

    let (emoji, text) = match penalty_event.outcome {
        gegen_data::types::PenaltyOutcome::Saved => {
            ("❌", format!("{}: Saved", penalty_event.player_name))
        }
        gegen_data::types::PenaltyOutcome::Scored => {
            ("✅", format!("{}: Scored", penalty_event.player_name))
        }
        gegen_data::types::PenaltyOutcome::Missed => {
            ("❌", format!("{}: Missed", penalty_event.player_name))
        }
    };

    (emoji, Text::from(text), event_side)
}

fn build_var_event(
    var_event: &VAREvent,
    home_team_id: Option<&String>,
) -> (&'static str, Text<'static>, EventSide) {
    let event_side = if Some(&var_event.team_id) == home_team_id {
        EventSide::Home
    } else {
        EventSide::Away
    };

    let text = format!(
        "{} ({}) -> {}: {}",
        var_event.var_type,
        var_event.player_name,
        var_event.decision,
        var_event.outcome.clone().unwrap_or_default(),
    );

    ("🔎", Text::from(text), event_side)
}

fn build_card_event(
    card_event: &CardEvent,
    home_team_id: Option<&String>,
) -> (&'static str, Text<'static>, EventSide) {
    let event_side = if Some(&card_event.team_id) == home_team_id {
        EventSide::Home
    } else {
        EventSide::Away
    };
    let emoji = match card_event.card_type {
        Card::Yellow => "🟨",
        Card::SecondYellow => "🟨🟨 (🟥)",
        Card::Red => "🟥",
    };
    let player_name = &card_event.player_name.clone().unwrap_or_default();

    let text = if let Some(reason) = &card_event.reason {
        format!("{} ({})", player_name, reason)
    } else {
        format!("{} ", player_name)
    };

    (emoji, Text::from(text), event_side)
}

fn build_sub_event(
    sub_event: &SubEvent,
    home_team_id: Option<&String>,
) -> (&'static str, Text<'static>, EventSide) {
    let event_side = if Some(&sub_event.team_id) == home_team_id {
        EventSide::Home
    } else {
        EventSide::Away
    };

    let text = format!(
        "On: {}, Off: {}",
        sub_event.player_name, sub_event.player2_name
    );

    ("🔄", Text::from(text), event_side)
}

fn build_goal_event(
    goal_event: &GoalEvent,
    home_team_id: Option<&String>,
) -> (&'static str, Text<'static>, EventSide) {
    let (text, event_side) = match goal_event.goal_type {
        GoalType::Goal => {
            let event_side = if Some(&goal_event.team_id) == home_team_id {
                EventSide::Home
            } else {
                EventSide::Away
            };

            (format_goal_text(goal_event), event_side)
        }
        GoalType::Penalty => {
            let event_side = if Some(&goal_event.team_id) == home_team_id {
                EventSide::Home
            } else {
                EventSide::Away
            };
            (format_goal_text(goal_event), event_side)
        }
        GoalType::OwnGoal => {
            let event_side = if Some(&goal_event.team_id) == home_team_id {
                EventSide::Away
            } else {
                EventSide::Home
            };

            (format_goal_text(goal_event), event_side)
        }
    };
    ("⚽", Text::from(text).yellow().italic(), event_side)
}

fn format_goal_text(goal_event: &GoalEvent) -> String {
    let prefix = match goal_event.goal_type {
        GoalType::Goal => "Goal",
        GoalType::Penalty => "Goal (p)",
        GoalType::OwnGoal => "Goal (og)",
    };
    if let Some(player_2_name) = &goal_event.player_2_name {
        format!(
            "{prefix}: {}, Assist: {}",
            goal_event.player_name, player_2_name
        )
    } else {
        format!("{prefix}: {}", goal_event.player_name)
    }
}

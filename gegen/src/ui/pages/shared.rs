use chrono::{Datelike, NaiveDate};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Stylize,
    widgets::Paragraph,
};
use throbber_widgets_tui::ThrobberState;

pub(in crate::ui::pages) fn render_title(
    frame: &mut Frame,
    area: Rect,
    date: &NaiveDate,
    todays_date: &NaiveDate,
    tab_title: String,
) {
    let layout = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Percentage(50),
        Constraint::Percentage(50),
        Constraint::Min(1),
    ]);
    let [_, left_area, right_area, _] = layout.areas(area);
    let weekday = date.weekday();
    let title = if date == todays_date {
        format!("Today ({weekday} - {date})")
    } else {
        format!("{weekday} - {date}")
    };

    frame.render_widget(
        Paragraph::new(title)
            .light_green()
            .alignment(Alignment::Left)
            .bold(),
        left_area,
    );

    frame.render_widget(
        Paragraph::new(tab_title)
            .light_green()
            .alignment(Alignment::Right)
            .magenta()
            .bold(),
        right_area,
    );
}

// TODO: make throbber render in centre of page
pub(in crate::ui::pages) fn render_loading(
    frame: &mut Frame,
    area: Rect,
    throbber_state: &mut ThrobberState,
) {
    let full = throbber_widgets_tui::Throbber::default()
        .label("loading...")
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
        .throbber_style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Magenta)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .throbber_set(throbber_widgets_tui::ARROW)
        .use_type(throbber_widgets_tui::WhichUse::Spin);
    frame.render_stateful_widget(full, area, throbber_state);
}

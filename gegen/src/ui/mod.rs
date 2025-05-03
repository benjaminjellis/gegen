use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph},
};

mod pages;
use crate::{
    GEGEN_VERSION,
    state::{Page, State},
};

pub(crate) fn process_event(event: Event, app_state: &mut State) {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') => app_state.should_quit = true,
            KeyCode::Char('m') => app_state.toggle_metadata_pop_up(),
            KeyCode::Char('?') => app_state.toggle_key_bind_pop_up(),
            _ => (),
        }
    }

    match app_state.current_page {
        Page::Matches(_) => {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('n') => app_state.next_day(),
                    KeyCode::Char('p') => app_state.previous_day(),
                    KeyCode::Char('t') => app_state.reset_to_today(),
                    KeyCode::Char('g') => app_state.page_states.live_scores.reset_scroll_state(),
                    KeyCode::Tab => {
                        let Some(grouped_data) = app_state.get_grouped_data() else {
                            return;
                        };
                        // subtract one here because tabs are zero indexed
                        let max_no_tabs = grouped_data.len() - 1;

                        app_state.page_states.live_scores.selected_tab = app_state
                            .page_states
                            .live_scores
                            .selected_tab
                            .saturating_add(1)
                            .min(max_no_tabs);
                        app_state.page_states.live_scores.table_state = Default::default();
                    }
                    KeyCode::BackTab => {
                        app_state.page_states.live_scores.selected_tab = app_state
                            .page_states
                            .live_scores
                            .selected_tab
                            .saturating_sub(1);

                        app_state.page_states.live_scores.table_state = Default::default();
                    }
                    _ => (),
                }
            }
        }
        Page::MatchOverview(..) => todo!(),
    }
}

pub(crate) fn draw_page(frame: &mut Frame, app_state: &mut State) {
    match app_state.current_page {
        Page::Matches(date) => pages::live_scores::draw(frame, app_state, &date),
        Page::MatchOverview(..) => unimplemented!(),
    }

    if app_state.show_metadata_pop_up {
        let block = Block::bordered()
            .title("Metadata")
            .title_style(Style::new().red());
        let paragraph = Paragraph::new(vec![
            Line::raw(format!("version: {GEGEN_VERSION}")),
            Line::raw("github: https://github.com/benjaminjellis/gegen"),
        ]);
        let area = popup_area(frame.area(), 60, 20);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph.block(block), area);
    }

    if app_state.show_key_bind_pop_up {
        let block = Block::bordered()
            .title("Key binds")
            .title_style(Style::new().red());

        let paragraph = Paragraph::new(vec![
            Line::raw(format!("version: {GEGEN_VERSION}")),
            Line::raw("github: https://github.com/benjaminjellis/gegen"),
        ]);

        let area = popup_area(frame.area(), 60, 20);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph.block(block), area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

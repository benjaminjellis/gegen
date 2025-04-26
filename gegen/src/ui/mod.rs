use crossterm::event::{Event, KeyCode};
use ratatui::Frame;

mod pages;
use crate::state::{Page, State};

pub(crate) fn process_event(event: Event, app_state: &mut State) {
    if let Event::Key(key) = event {
        app_state.should_quit = KeyCode::Char('q') == key.code;
    }
    match app_state.current_page {
        Page::Matches(_) => {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('n') => app_state.next_day(),
                    KeyCode::Char('p') => app_state.previous_day(),
                    KeyCode::Char('t') => app_state.reset_to_today(),
                    _ => (),
                }
            }
        }
        Page::MatchOverview => todo!(),
    }
}

pub(crate) fn draw_page(frame: &mut Frame, app_state: &mut State) {
    match app_state.current_page {
        Page::Matches(date) => pages::live_scores::draw(frame, app_state, &date),
        Page::MatchOverview => unimplemented!(),
    }
}

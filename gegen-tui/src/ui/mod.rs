use crossterm::event::{Event, KeyCode};
use ratatui::Frame;

mod pages;
use crate::state::{Page, State};

pub(crate) fn process_event(event: Event, app_state: &mut State) {
    if let Event::Key(key) = event {
        app_state.should_quit = KeyCode::Char('q') == key.code;
    }
}

pub(crate) fn draw_page(frame: &mut Frame, app_state: &mut State) {
    match &app_state.current_page {
        Page::LiveScores => pages::live_scores::draw(frame, app_state),
    }
}

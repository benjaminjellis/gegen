use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use gegen_data::types::LiveScoresResponse;

pub(crate) type LiveData = Arc<RwLock<Option<LiveScoresResponse>>>;

pub(crate) enum Page {
    LiveScores,
}

pub(crate) struct State {
    pub(crate) data: LiveData,
    tick_rate: Duration,
    last_tick: SystemTime,
    pub(crate) current_page: Page,
    pub(crate) page_states: PageStates,
    pub(crate) should_quit: bool,
}

#[derive(Default)]
pub(crate) struct PageStates {
    pub(crate) live_scores: LiveScoresPageState,
}

#[derive(Default)]
pub(crate) struct LiveScoresPageState {
    pub(crate) throbber_state: throbber_widgets_tui::ThrobberState,
}

impl State {
    pub(crate) fn new(data: LiveData) -> Self {
        Self {
            data,
            tick_rate: Duration::from_millis(150),
            last_tick: SystemTime::now(),
            current_page: Page::LiveScores,
            should_quit: false,
            page_states: PageStates::default(),
        }
    }

    pub(crate) fn should_draw(&mut self) -> bool {
        if self.last_tick.elapsed().unwrap() > self.tick_rate {
            self.last_tick = SystemTime::now();
            true
        } else {
            false
        }
    }

    pub(crate) fn on_tick(&mut self) {
        self.page_states.live_scores.throbber_state.calc_next();
    }
}

use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use chrono::{Days, Local, NaiveDate};
use crossbeam::channel::Sender;
use dashmap::DashMap;
use gegen_data::types::LiveScoresResponse;

pub(crate) type LiveData = Arc<DashMap<NaiveDate, LiveScoresResponse>>;

pub(crate) enum Page {
    Matches(NaiveDate),
    MatchOverview,
}

pub(crate) struct State {
    pub(crate) data: LiveData,
    tick_rate: Duration,
    last_tick: SystemTime,
    pub(crate) current_page: Page,
    pub(crate) page_states: PageStates,
    pub(crate) should_quit: bool,
    sender: Sender<NaiveDate>,
    pub(crate) today: NaiveDate,
    pub(crate) show_metadata_pop_up: bool,
}

#[derive(Default)]
pub(crate) struct PageStates {
    pub(crate) live_scores: LiveScoresPageState,
}

#[derive(Default)]
pub(crate) struct LiveScoresPageState {
    pub(crate) throbber_state: throbber_widgets_tui::ThrobberState,
    pub(crate) vertical_scroll: usize,
}

impl LiveScoresPageState {
    pub(crate) fn reset_scroll_state(&mut self) {
        self.vertical_scroll = 0;
    }
}

impl State {
    pub(crate) fn new(data: LiveData, sender: Sender<NaiveDate>) -> Self {
        let today = get_todays_date();
        Self {
            data,
            tick_rate: Duration::from_millis(150),
            last_tick: SystemTime::now(),
            current_page: Page::Matches(today),
            should_quit: false,
            page_states: PageStates::default(),
            sender,
            today,
            show_metadata_pop_up: false,
        }
    }

    pub(crate) fn scroll_down(&mut self) {
        match self.current_page {
            Page::Matches(_) => {
                self.page_states.live_scores.vertical_scroll = self
                    .page_states
                    .live_scores
                    .vertical_scroll
                    .saturating_add(1);
            }
            Page::MatchOverview => (),
        }
    }

    pub(crate) fn scroll_up(&mut self) {
        match self.current_page {
            Page::Matches(_) => {
                self.page_states.live_scores.vertical_scroll = self
                    .page_states
                    .live_scores
                    .vertical_scroll
                    .saturating_sub(1);
            }
            Page::MatchOverview => (),
        }
    }

    pub(crate) fn toggle_metadata_pop_up(&mut self) {
        self.show_metadata_pop_up = !self.show_metadata_pop_up;
    }

    pub(crate) fn reset_to_today(&mut self) {
        self.current_page = Page::Matches(self.today);
    }

    pub(crate) fn previous_day(&mut self) {
        let Page::Matches(current_page_date) = self.current_page else {
            return;
        };

        let next_day = current_page_date.checked_sub_days(Days::new(1)).unwrap();
        self.current_page = Page::Matches(next_day);
        self.page_states.live_scores.reset_scroll_state();
    }

    pub(crate) fn next_day(&mut self) {
        let Page::Matches(current_page_date) = self.current_page else {
            return;
        };

        let next_day = current_page_date.checked_add_days(Days::new(1)).unwrap();
        self.current_page = Page::Matches(next_day);
        self.page_states.live_scores.reset_scroll_state();
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
        self.today = get_todays_date();
    }

    pub(crate) fn fetch_data_for_date(&self, date: NaiveDate) {
        if let Err(err) = self.sender.try_send(date) {
            tracing::error!("failed to send date to data fetch thread: {err}")
        }
    }
}

fn get_todays_date() -> NaiveDate {
    Local::now().date_naive()
}

use std::time::Duration;

use gegen_data::get_live_scores;

use crate::state::LiveData;

const FETCH_DELAY: Duration = Duration::from_secs(4);

fn fetch_data(data: LiveData) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();
    loop {
        // tracing::info!("getting live scores");
        let live_scores = get_live_scores(&client).unwrap();
        // tracing::info!("got live scores");
        let mut data_state = data.write().unwrap();
        *data_state = Some(live_scores);
        drop(data_state);
        std::thread::sleep(FETCH_DELAY);
    }
}

pub(crate) fn run_data_fetch(data: &LiveData) -> std::thread::JoinHandle<Result<(), String>> {
    // run the data collection thread in the background
    std::thread::Builder::new()
        .name("data fetch thread".into())
        .spawn({
            let data = data.clone();
            move || fetch_data(data)
        })
        .unwrap()
}

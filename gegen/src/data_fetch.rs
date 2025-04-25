use std::time::Duration;

use gegen_data::get_live_scores;

use crate::state::LiveData;

use color_eyre::eyre::Result;

const FETCH_DELAY: Duration = Duration::from_secs(4);
const DATA_FETCH_THREAD_NAME: &str = "data fetch thread";

fn fetch_data(data: LiveData) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let mut failure_count = 0;
    loop {
        // tracing::info!("getting live scores");
        match get_live_scores(&client) {
            Ok(live_scores) => match data.write() {
                Ok(mut data) => {
                    *data = Some(live_scores);
                    drop(data);
                    failure_count = 0;
                }
                Err(err) => {
                    tracing::error!("got error attempting to take write: {err}");
                    failure_count += 1;
                }
            },
            Err(err) => {
                failure_count += 1;
                tracing::error!("got error when fetching data: {err}");
            }
        };

        if failure_count >= 3 {
            tracing::error!("encountered three consecutive failutres to fetch data, aborting");
        }

        std::thread::sleep(FETCH_DELAY);
    }
}

// run the data collection thread in the background
pub(crate) fn run_data_fetch(data: &LiveData) -> std::thread::JoinHandle<Result<()>> {
    std::thread::Builder::new()
        .name(DATA_FETCH_THREAD_NAME.into())
        .spawn({
            let data = data.clone();
            move || fetch_data(data)
        })
        .expect("Failed to {DATA_FETCH_THREAD_NAME} thread")
}

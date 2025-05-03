use std::time::{Duration, SystemTime};

use chrono::NaiveDate;
use crossbeam::channel::Receiver;
use gegen_data::{get_live_scores, get_matches};

use crate::state::LiveData;

const FETCH_DELAY: Duration = Duration::from_secs(4);
const DATA_FETCH_THREAD_NAME: &str = "data fetch thread";
const SLEEP: Duration = Duration::from_millis(100);

fn fetch_data(data: LiveData, current_date: NaiveDate, recv: Receiver<NaiveDate>) {
    let client = reqwest::blocking::Client::new();
    let mut failure_count = 0;
    let mut last_fetched_live_date = SystemTime::now();

    // prefetch
    fetch_and_insert_data(
        &client,
        &data,
        current_date,
        &mut failure_count,
        DataToFetch::Live,
    );

    loop {
        if let Ok(other_date) = recv.try_recv() {
            tracing::info!("fetching data for {other_date}");
            fetch_and_insert_data(
                &client,
                &data,
                other_date,
                &mut failure_count,
                DataToFetch::Fixtures,
            );
        }

        match last_fetched_live_date.elapsed() {
            Ok(elapsed_since_last_fetch) => {
                if elapsed_since_last_fetch > FETCH_DELAY {
                    tracing::info!("FETCH");
                    fetch_and_insert_data(
                        &client,
                        &data,
                        current_date,
                        &mut failure_count,
                        DataToFetch::Live,
                    );
                    last_fetched_live_date = SystemTime::now();
                }
            }
            Err(err) => {
                tracing::error!(
                    "failed to calculate time elapsed since last fetch of live data: {err}"
                );
            }
        }

        if failure_count >= 3 {
            tracing::error!("encountered three consecutive failutres to fetch data, aborting");
            break;
        }

        // HACK: try_recv returns imeditealy so without a sleep this loop is a busy-wait and thus chews
        // through cpu cycles. Sleeping fixes that so the thread yields and the OS can go a spend
        // cpu cycles elsewhere.
        std::thread::sleep(SLEEP);
    }
}

fn fetch_and_insert_data(
    client: &reqwest::blocking::Client,
    data: &LiveData,
    date: NaiveDate,
    failure_count: &mut u32,
    data_to_fetch: DataToFetch,
) {
    let response = match data_to_fetch {
        DataToFetch::Live => get_live_scores(client),
        DataToFetch::Fixtures => get_matches(client, date),
    };
    match response {
        Ok(live_scores) => {
            data.insert(date, live_scores);

            *failure_count = 0;
        }
        Err(err) => {
            tracing::error!("got error when fetching data: {err}");

            *failure_count += 1;
        }
    }
}

enum DataToFetch {
    Live,
    Fixtures,
}

// run the data collection thread in the background
pub(crate) fn run_data_fetch(
    data: &LiveData,
    current_date: NaiveDate,
    recv: Receiver<NaiveDate>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name(DATA_FETCH_THREAD_NAME.into())
        .spawn({
            let data = data.clone();
            move || fetch_data(data, current_date, recv)
        })
        .expect("Failed to run thread: {DATA_FETCH_THREAD_NAME}")
}

use chrono::NaiveDate;
use const_format::concatcp;
use reqwest::StatusCode;
use types::LiveScoresResponse;

pub mod types;
mod utils;

const BASE_URL: &str = "https://optaplayerstats.statsperform.com/api/";
const LIVE_SCORE_URL: &str = concatcp!(BASE_URL, "en_GB/soccer/livescores/");
const MATTCHES_URL: &str = concatcp!(BASE_URL, "en_GB/soccer/matches/");

pub fn get_live_scores(
    client: &reqwest::blocking::Client,
) -> Result<LiveScoresResponse, GegenDataError> {
    let live_score_query_params = types::LiveScoreQueryParams { offset: 0 };

    let headers = utils::create_header_maps();
    let resp = client
        .get(LIVE_SCORE_URL)
        .query(&live_score_query_params)
        .headers(headers)
        .send()
        .map_err(|source| GegenDataError::Reqwest {
            source,
            url: LIVE_SCORE_URL.to_string(),
        })?;

    match resp.status() {
        StatusCode::OK => {
            resp.json::<LiveScoresResponse>()
                .map_err(|source| GegenDataError::Serialisation {
                    source,
                    url: LIVE_SCORE_URL.to_string(),
                })
        }
        StatusCode::TOO_MANY_REQUESTS => Err(GegenDataError::TooManyRequests {
            url: LIVE_SCORE_URL.to_string(),
        }),
        other_status_code => Err(GegenDataError::Non200 {
            status_code: other_status_code,
            url: LIVE_SCORE_URL.to_string(),
            body: resp.text().unwrap_or_default(),
        }),
    }
}

pub fn get_matches(
    client: &reqwest::blocking::Client,
    date: NaiveDate,
) -> Result<LiveScoresResponse, GegenDataError> {
    let url = format!("{MATTCHES_URL}{date}");
    let live_score_query_params = types::LiveScoreQueryParams { offset: 0 };

    let headers = utils::create_header_maps();
    let resp = client
        .get(&url)
        .query(&live_score_query_params)
        .headers(headers)
        .send()
        .map_err(|source| GegenDataError::Reqwest {
            source,
            url: url.clone(),
        })?;

    match resp.status() {
        StatusCode::OK => resp
            .json::<LiveScoresResponse>()
            .map_err(|source| GegenDataError::Serialisation { source, url }),
        StatusCode::TOO_MANY_REQUESTS => Err(GegenDataError::TooManyRequests { url }),
        other_status_code => Err(GegenDataError::Non200 {
            status_code: other_status_code,
            url,
            body: resp.text().unwrap_or_default(),
        }),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GegenDataError {
    #[error("Failed to send request to {url}: {source}")]
    Reqwest { source: reqwest::Error, url: String },
    #[error("Got a 429 / too many rqeusts {url}")]
    TooManyRequests { url: String },
    #[error("Got a 429 / too many rqeusts {url}")]
    Non200 {
        status_code: StatusCode,
        url: String,
        body: String,
    },
    #[error("Failed to deserialise response for {url}: {source:?}")]
    Serialisation { source: reqwest::Error, url: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_scores() {
        let client = reqwest::blocking::Client::new();
        let _ = get_live_scores(&client).unwrap();
    }

    #[test]
    fn test_fixtures() {
        let date = NaiveDate::from_ymd_opt(2025, 4, 27).unwrap();

        let client = reqwest::blocking::Client::new();

        let _ = get_matches(&client, date).unwrap();
    }
}

use chrono::{DateTime, Utc, serde::ts_seconds};
use const_format::concatcp;
use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://optaplayerstats.statsperform.com/api/";
const LIVE_SCORE_URL: &str = concatcp!(BASE_URL, "ro_RO/soccer/livescores");

#[derive(Serialize)]
struct LiveScoreQueryParams {
    offset: u8,
}

fn create_header_maps() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert("Accept-Encoding", "".parse().unwrap());

    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Host", "optaplayerstats.statsperform.com".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());

    headers.insert(
        "Referer",
        "https://optaplayerstats.statsperform.com/ro_RO/soccer"
            .parse()
            .unwrap(),
    );

    headers.insert(
        "User-Agent",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:137.0) Gecko/20100101 Firefox/137.0"
            .parse()
            .unwrap(),
    );
    headers
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiveScoresResponse {
    matches: Vec<Match>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Match {
    id: String,
    status: Status,
    comp: Competition,
    #[serde(with = "ts_seconds")]
    date: DateTime<Utc>,
    home: Team,
    away: Team,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Competition {
    id: String,
    name: String,
    country: Country,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Country {
    id: String,
    full_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Team {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Played,
    Fixture,
    Playing,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let live_score_query_params = LiveScoreQueryParams { offset: 0 };
    let client = Client::new();
    let headers = create_header_maps();

    dbg!(&LIVE_SCORE_URL);
    let req = client
        .get(LIVE_SCORE_URL)
        .query(&live_score_query_params)
        .headers(headers);

    let scores = req.send().await.unwrap();
    let body = scores.json::<LiveScoresResponse>().await.unwrap();
    dbg!(body);
}

use chrono::NaiveDate;
use const_format::concatcp;
use reqwest::StatusCode;
use types::LiveScoresResponse;

mod types;
mod utils;

const BASE_URL: &str = "https://optaplayerstats.statsperform.com/api/";
const LIVE_SCORE_URL: &str = concatcp!(BASE_URL, "ro_RO/soccer/livescores");
const MATTCHES_URL: &str = concatcp!(BASE_URL, "ro_RO/soccer/matches/");

pub async fn get_live_score(client: &reqwest::Client) -> LiveScoresResponse {
    let live_score_query_params = types::LiveScoreQueryParams { offset: 0 };

    let headers = utils::create_header_maps();
    let resp = client
        .get(LIVE_SCORE_URL)
        .query(&live_score_query_params)
        .headers(headers)
        .send()
        .await
        .unwrap();

    match resp.status() {
        StatusCode::OK => resp.json::<LiveScoresResponse>().await.unwrap(),
        StatusCode::TOO_MANY_REQUESTS => todo!(),
        other => todo!(),
    }
}

pub async fn get_matches(client: &reqwest::Client, date: NaiveDate) -> LiveScoresResponse {
    let url = format!("{MATTCHES_URL}/{date}");
    let live_score_query_params = types::LiveScoreQueryParams { offset: 0 };

    let headers = utils::create_header_maps();
    let resp = client
        .get(url)
        .query(&live_score_query_params)
        .headers(headers)
        .send()
        .await
        .unwrap();

    match resp.status() {
        StatusCode::OK => resp.json::<LiveScoresResponse>().await.unwrap(),
        StatusCode::TOO_MANY_REQUESTS => todo!(),
        other => todo!(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_live_scores() {
        let client = reqwest::Client::new();
        let scores = get_live_score(&client).await;
        dbg!(scores);
    }
}

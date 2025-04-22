use std::collections::HashMap;

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
pub struct LiveScoresResponse {
    pub matches: Vec<Match>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub id: String,
    pub status: Status,
    pub comp: Competition,
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
    pub home: Team,
    pub away: Team,
    pub score: HashMap<ScoreKey, Score>,
    pub events: Vec<Event>,

    #[serde(with = "ts_seconds")]
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "entity_type")]
pub enum Event {
    Sub(SubEvent),
    Goal(GoalEvent),
    Card(CardEvent),
    Var,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardEvent {
    pub period_id: u8,
    pub min: u16,
    pub time_str: String,
    pub team_id: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
}

#[derive(Debug, Deserialize)]
pub enum CardType {
    #[serde(rename = "YC")]
    YellowCard,
    #[serde(rename = "Y2C")]
    SecondYellowCard,
    #[serde(rename = "RC")]
    RedCard,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalEvent {
    pub period_id: u8,
    pub min: u16,
    pub time_str: String,
    pub team_id: String,
    pub player_id: String,
    pub player_name: String,
    #[serde(rename = "type")]
    pub goal_type: GoalType,
    pub score: [u8; 2],
}

#[derive(Debug, Deserialize)]
pub enum GoalType {
    #[serde(rename = "G")]
    Goal,
    #[serde(rename = "PG")]
    Penalty,
    #[serde(rename = "OG")]
    OwnGoal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubEvent {
    pub period_id: u8,
    pub min: u16,
    pub time_str: String,
    pub team_id: String,
    pub player_id: String,
    pub player_name: String,
    pub player2_id: String,
    pub player2_name: String,
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScoreKey {
    Ft,
    Ht,
    Total,
    Aggregate,
    TotalUnconfirmed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub home: u8,
    pub away: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Competition {
    pub id: String,
    pub name: String,
    pub country: Country,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub id: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
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

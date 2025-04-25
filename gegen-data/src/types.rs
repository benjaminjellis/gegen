use std::collections::HashMap;

use chrono::{DateTime, Utc, serde::ts_seconds};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct LiveScoreQueryParams {
    pub(crate) offset: u8,
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
    pub score: Option<HashMap<ScoreKey, Score>>,
    pub events: Option<Vec<Event>>,
    #[serde(with = "ts_seconds")]
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "entity_type")]
pub enum Event {
    Sub(SubEvent),
    Goal(GoalEvent),
    Card(CardEvent),
    Var(VAREvent),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VAREvent {
    pub period_id: u8,
    pub min: u16,
    pub time_str: String,
    pub team_id: String,
    pub player_id: String,
    pub player_name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub outcome: String,
    pub decision: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardEvent {
    pub period_id: u8,
    pub min: u16,
    pub time_str: String,
    pub team_id: String,
    #[serde(rename = "type")]
    pub card_type: Card,
}

#[derive(Debug, Deserialize)]
pub enum Card {
    #[serde(rename = "YC")]
    Yellow,
    #[serde(rename = "Y2C")]
    SecondYellow,
    #[serde(rename = "RC")]
    Red,
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
    pub score: Option<[u8; 2]>,
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

#[derive(Debug, Deserialize, Clone)]
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

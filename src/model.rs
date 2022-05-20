use serde::Deserialize;
use std::collections::HashMap;

pub type History = HashMap<LanguageDirection, Vec<Card>>;
pub type LanguageDirection = String;

#[derive(Deserialize, Debug)]
pub struct Card {
    pub spelling: String,
    pub reading: String,
    pub vid: u64,
    pub reviews: Vec<CardEvent>,
}

#[derive(Deserialize, Debug)]
pub struct CardEvent {
    pub timestamp: i64,
    pub grade: String,
}

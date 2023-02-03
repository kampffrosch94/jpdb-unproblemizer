use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct History {
    pub cards_vocabulary_jp_en: Vec<CardVocabularyJpEn>,
}

#[derive(Deserialize, Debug)]
pub struct CardVocabularyJpEn {
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

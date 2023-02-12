use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct History {
    pub cards_vocabulary_jp_en: Vec<CardVocabularyJpEn>,
    pub cards_kanji_char_keyword: Vec<CardKanjiCharKeyword>
}

#[derive(Deserialize, Debug)]
pub struct CardVocabularyJpEn {
    pub spelling: String,
    pub reading: String,
    pub vid: u64,
    pub reviews: Vec<CardEvent>,
}


#[derive(Deserialize, Debug)]
pub struct CardKanjiCharKeyword {
    pub character: String,
    pub reviews: Vec<CardEvent>,
}

#[derive(Deserialize, Debug)]
pub struct CardEvent {
    pub timestamp: i64,
    pub grade: String,
}

mod model;

use crate::model::*;
use anyhow::{anyhow, bail, Context, Result};
use chrono::{Duration, TimeZone, Utc};
use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use std::path::Path;
use std::thread::sleep;

const DOMAIN: &str = "jpdb.io";
const URL_PREFIX: &str = "https://";
const COOKIE_NAME: &str = "sid";

fn main() -> Result<()> {
    let dry_run = true;
    println!("Program start.");

    let jar = Jar::default();
    let cookie_content = std::fs::read_to_string("cookie").unwrap();
    let cookie_str = format!("{COOKIE_NAME}={cookie_content}; Domain={DOMAIN}");
    jar.add_cookie_str(&cookie_str, &format!("{URL_PREFIX}{DOMAIN}").parse()?);
    let client = Client::builder()
        .cookie_store(true)
        .cookie_provider(jar.into())
        .build()?;

    let fname = "history.json";
    let history_text = if Path::new(fname).exists() {
        println!("Using local history");
        std::fs::read_to_string(fname)?
    } else {
        println!("Fetching remote history");
        let url = "https://jpdb.io/export/vocabulary-reviews.json";
        let req = client.get(url).build()?;
        let body = client.execute(req)?.text()?;
        std::fs::write(fname, &body)?;
        body
    };

    let failure_states = ["nothing", "something", "fail"];

    let current_time = Utc::now();
    let history: model::History = serde_json::from_str(&history_text)?;
    let bad_cards = || {
        history
            .cards_vocabulary_jp_en
            .iter()
            .filter(|card: &&CardVocabularyJpEn| {
                card.reviews
                    .iter()
                    .filter(|ev: &&CardEvent| {
                        let ts = Utc.timestamp(ev.timestamp, 0);
                        current_time - ts <= Duration::days(1)
                            && failure_states.contains(&ev.grade.as_str())
                    })
                    .count()
                    >= 3
            })
    };

    if dry_run {
        println!("Bad cards");
        for card in bad_cards() {
            let history_url = format!(
                "https://jpdb.io/vocabulary/{}/{}/review-history",
                card.vid, card.spelling
            );
            println!("{}", history_url);
            open::that(history_url)?;
            sleep(std::time::Duration::from_millis(500));
        }
        return Ok(());
    }

    for card in bad_cards() {
        println!("Erasing history of {}", card.spelling);
        let history_url = format!(
            "https://jpdb.io/vocabulary/{}/{}/review-history",
            card.vid, card.spelling
        );
        let req = client.get(history_url).build()?;
        let body = client.execute(req)?.text()?;

        let document = scraper::Html::parse_document(&body);
        let mut payload: Vec<(String, String)> = Vec::new();
        for field_name in ["v", "s", "origin"] {
            let selector = scraper::Selector::parse(&format!(r#"input[name="{}"]"#, field_name))
                .map_err(|e| anyhow!("{e:?}"))?;
            let selected = document
                .select(&selector)
                .next()
                .context("no element found")?;
            let value = selected
                .value()
                .attr("value")
                .context("no value on element")?;
            println!("{}={}", field_name, value);
            payload.push((field_name.into(), value.into()))
        }

        let delete_url = "https://jpdb.io/clear-review-history";
        let req = client.post(delete_url).form(&payload).build()?;
        if !client.execute(req)?.status().is_success() {
            bail!("Error executing clear query")
        }
    }

    Ok(())
}

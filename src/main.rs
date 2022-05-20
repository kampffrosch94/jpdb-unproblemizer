mod model;

use crate::model::{Card, CardEvent};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{Duration, TimeZone, Utc};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use std::path::Path;

const DOMAIN: &str = "jpdb.io";
const URL_PREFIX: &str = "https://";
const COOKIE_NAME: &str = "sid";

lazy_static! {
    static ref CLIENT: Client = {
        let jar = Jar::default();
        let cookie_content = std::fs::read_to_string("cookie").unwrap();
        let cookie_str = format!("{COOKIE_NAME}={cookie_content}; Domain={DOMAIN}");
        jar.add_cookie_str(
            &cookie_str,
            &format!("{URL_PREFIX}{DOMAIN}").parse().unwrap(),
        );
        Client::builder()
            .cookie_store(true)
            .cookie_provider(jar.into())
            .build()
            .unwrap()
    };
}

fn main() -> Result<()> {
    println!("Let's go.");

    let fname = "history.json";
    let history_text = if Path::new(fname).exists() {
        println!("Using local history");
        std::fs::read_to_string(fname)?
    } else {
        println!("Fetching remote history");
        let url = "https://jpdb.io/export/vocabulary-reviews.json";
        let req = CLIENT.get(url).build()?;
        let body = CLIENT.execute(req)?.text()?;
        std::fs::write(fname, &body)?;
        body
    };

    let failure_states = ["nothing", "something", "fail"];

    let current_time = Utc::now();
    let history: model::History = serde_json::from_str(&history_text)?;
    let bad_cards = history.values().flatten().filter(|card: &&Card| {
        card.reviews
            .iter()
            .filter(|ev: &&CardEvent| {
                let ts = Utc.timestamp(ev.timestamp, 0);
                current_time - ts <= Duration::days(7)
                    && failure_states.contains(&ev.grade.as_str())
            })
            .count()
            >= 7
    });

    for card in bad_cards {
        println!("Erasing history of {}", card.spelling);
        let history_url = format!(
            "https://jpdb.io/vocabulary/{}/{}/review-history",
            card.vid, card.spelling
        );
        let req = CLIENT.get(history_url).build()?;
        let body = CLIENT.execute(req)?.text()?;

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
        let req = CLIENT.post(delete_url).form(&payload).build()?;
        if !CLIENT.execute(req)?.status().is_success() {
            bail!("Error executing clear query")
        }
    }

    Ok(())
}

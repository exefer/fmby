use crate::constants::FMHY_SINGLE_PAGE_ENDPOINT;
use crate::utils::db::ChunkSize;
use entity::sea_orm_active_enums::WikiUrlStatus;
use entity::wiki_urls::{self, Entity as WikiUrls};
use migration::OnConflict;
use pulldown_cmark::{Event, Tag, TagEnd};
use regex::Regex;
use sea_orm::ActiveValue::*;
use sea_orm::TransactionTrait;
use sea_orm::prelude::*;

#[derive(Debug, Clone)]
pub struct WikiLink {
    url: String,
    name: Option<String>,
}

impl WikiLink {
    fn into_active_model(self) -> wiki_urls::ActiveModel {
        wiki_urls::ActiveModel {
            url: Set(self.url),
            name: Set(self.name),
            status: Set(WikiUrlStatus::Added),
            ..Default::default()
        }
    }
}

pub fn clean_url(url: &str) -> &str {
    url.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches('/')
}

pub fn extract_urls(haystack: &str) -> Option<Vec<String>> {
    let regex = Regex::new(r"(https?):\/\/(?:ww(?:w|\d+)\.)?((?:[\w_-]+(?:\.[\w_-]+)+)[\w.,@?^=%&:\/~+#-]*[\w@?^=%&~+-])").unwrap();

    let matches: Vec<String> = regex
        .find_iter(haystack)
        .map(|m| clean_url(m.as_str()).to_string())
        .collect();

    if matches.is_empty() {
        None
    } else {
        Some(matches)
    }
}

pub async fn fetch_wiki_links() -> anyhow::Result<Vec<WikiLink>> {
    let content = reqwest::get(FMHY_SINGLE_PAGE_ENDPOINT)
        .await?
        .text()
        .await?;
    let parser = pulldown_cmark::Parser::new(&content);

    let mut current_url = String::new();
    let mut current_name = String::new();
    let mut collecting = false;
    let mut links = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                current_url = dest_url.to_string();
                current_name.clear();
                collecting = true;
            }
            Event::End(TagEnd::Link) if collecting => {
                links.push(WikiLink {
                    url: clean_url(&current_url).to_string(),
                    name: if current_name.is_empty() {
                        None
                    } else {
                        Some(std::mem::take(&mut current_name))
                    },
                });
                collecting = false;
            }
            Event::Text(text) if collecting => current_name.push_str(&text),
            _ => {}
        }
    }

    Ok(links)
}

pub async fn insert_wiki_urls(
    conn: &DatabaseConnection,
    mut entries: Vec<WikiLink>,
) -> Result<(), DbErr> {
    let chunk_size = WikiUrls::chunk_size();

    while !entries.is_empty() {
        let chunk: Vec<_> = entries
            .drain(..chunk_size.min(entries.len()))
            .map(WikiLink::into_active_model)
            .collect();

        let tx = conn.begin().await?;

        WikiUrls::insert_many(chunk)
            .on_conflict(
                OnConflict::column(wiki_urls::Column::Url)
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec(&tx)
            .await?;

        tx.commit().await?;
    }

    Ok(())
}

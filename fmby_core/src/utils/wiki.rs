use crate::{
    constants::FMHY_SINGLE_PAGE_ENDPOINT,
    utils::{db::ChunkSize, url::clean_url},
};
use fmby_entities::{prelude::*, sea_orm_active_enums::WikiUrlStatus, wiki_urls};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use regex::Regex;
use sea_orm::{ActiveValue::*, TransactionTrait, prelude::*, sea_query::OnConflict};

pub async fn search_in_wiki(query: &str) -> anyhow::Result<Vec<String>> {
    let query = query.to_lowercase();
    let query_re = Regex::new(&format!("(?i){}", regex::escape(&query))).unwrap();

    let content = reqwest::get(FMHY_SINGLE_PAGE_ENDPOINT)
        .await?
        .text()
        .await?;

    let mut result = Vec::new();
    let mut current_headings = Vec::new();
    let mut heading_path = String::new();
    let mut parser_iter = Parser::new(&content).into_offset_iter();

    while let Some((event, range)) = parser_iter.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let heading_text = collect_heading_text(&mut parser_iter)
                    .replace(['►', '▷'], "")
                    .trim()
                    .to_string();
                let level = level as usize;
                if current_headings.len() >= level {
                    current_headings.truncate(level - 1);
                }
                current_headings.push(heading_text);
                heading_path = current_headings
                    .iter()
                    .map(|s| format!("**{}**", s))
                    .collect::<Vec<_>>()
                    .join(" / ");
            }
            Event::Start(Tag::Item) => {
                let line_start = range.start;
                let line_end = content[line_start..]
                    .find('\n')
                    .map(|i| line_start + i)
                    .unwrap_or(content.len());
                let line = &content[line_start..line_end];

                if let Some(stripped) = line.strip_prefix("* ")
                    && (query_re.is_match(stripped) || query_re.is_match(&heading_path))
                {
                    let mut formatted_line =
                        String::with_capacity(heading_path.len() + stripped.len() + 3);
                    formatted_line.push_str(&heading_path);
                    formatted_line.push_str(" ► ");
                    formatted_line.push_str(stripped);

                    result.push(formatted_line);
                }
            }
            _ => {}
        }
    }

    Ok(result)
}

fn collect_heading_text<'a, I>(parser: &mut I) -> String
where
    I: Iterator<Item = (Event<'a>, std::ops::Range<usize>)>,
{
    let mut text = String::new();
    for (event, _) in parser.by_ref() {
        match event {
            Event::Text(t) | Event::Code(t) => text.push_str(&t),
            Event::End(TagEnd::Heading(_)) => break,
            _ => {}
        }
    }
    text
}

pub struct WikiLink {
    url: String,
}

impl WikiLink {
    fn into_active_model(self) -> wiki_urls::ActiveModel {
        wiki_urls::ActiveModel {
            url: Set(self.url),
            status: Set(WikiUrlStatus::Added),
            ..Default::default()
        }
    }
}

pub async fn fetch_wiki_links() -> anyhow::Result<Vec<WikiLink>> {
    let content = reqwest::get(FMHY_SINGLE_PAGE_ENDPOINT)
        .await?
        .text()
        .await?;
    let parser = Parser::new(&content);
    let mut links = Vec::new();

    for event in parser {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            links.push(WikiLink {
                url: clean_url(&dest_url).to_string(),
            })
        }
    }

    Ok(links)
}

pub async fn insert_wiki_links(
    pool: &DatabaseConnection,
    mut entries: Vec<WikiLink>,
) -> Result<(), DbErr> {
    let chunk_size = WikiUrls::chunk_size();

    while !entries.is_empty() {
        let chunk: Vec<_> = entries
            .drain(..chunk_size.min(entries.len()))
            .map(WikiLink::into_active_model)
            .collect();
        let txn = pool.begin().await?;

        WikiUrls::insert_many(chunk)
            .on_conflict(
                OnConflict::column(wiki_urls::Column::Url)
                    .do_nothing()
                    .to_owned(),
            )
            .do_nothing()
            .exec(&txn)
            .await?;

        txn.commit().await?;
    }

    Ok(())
}

use crate::constants::FMHY_SINGLE_PAGE_ENDPOINT;
use pulldown_cmark::{CowStr, Event, Parser, Tag, TagEnd};
use regex::Regex;

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
                    .to_owned();
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
                    .map_or(content.len(), |i| line_start + i);
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

pub fn collect_wiki_urls<'a>(content: &'a str) -> Vec<CowStr<'a>> {
    let parser = Parser::new(content);
    let mut urls = Vec::new();

    for event in parser {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            urls.push(dest_url);
        }
    }

    urls
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

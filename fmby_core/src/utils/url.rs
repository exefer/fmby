use std::sync::LazyLock;

use regex::Regex;

pub fn clean_url(url: &str) -> &str {
    url.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches("?tab=readme-ov-file")
        .trim_end_matches('/')
}

static URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(https?):\/\/(?:ww(?:w|\d+)\.)?((?:[\w_-]+(?:\.[\w_-]+)+)[\w.,@?^=%&:\/~+#-]*[\w@?^=%&~+-])").unwrap()
});

pub fn extract_urls(haystack: &str) -> Option<Vec<String>> {
    let matches: Vec<String> = URL_RE
        .find_iter(haystack)
        .map(|m| clean_url(m.as_str()).to_owned())
        .filter(|s| !s.starts_with("discord.com/channels") && !s.starts_with("fmhy.net"))
        .collect();

    (!matches.is_empty()).then_some(matches)
}

use regex::Regex;
use std::sync::LazyLock;

pub fn clean_url(url: &str) -> &str {
    url.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .trim_end_matches('/')
}

static URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(https?):\/\/(?:ww(?:w|\d+)\.)?((?:[\w_-]+(?:\.[\w_-]+)+)[\w.,@?^=%&:\/~+#-]*[\w@?^=%&~+-])").unwrap()
});

pub fn extract_urls(haystack: &str, clean: bool) -> Option<Vec<String>> {
    let matches: Vec<String> = URL_RE
        .find_iter(haystack)
        .map(|m| {
            let url = m.as_str();
            if clean {
                clean_url(url).to_string()
            } else {
                url.to_string()
            }
        })
        .collect();

    (!matches.is_empty()).then_some(matches)
}

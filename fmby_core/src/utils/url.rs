use std::sync::LazyLock;

use regex::Regex;
use url::Url;

const TRACKING_PARAMS: &[&str] = &[
    "__hsfp",
    "__hssc",
    "__hstc",
    "__s",
    "__twitter_impression",
    "_ga",
    "_gat",
    "_gid",
    "_gl",
    "_hsenc",
    "_hsmi",
    "_openstat",
    "_pk_campaign",
    "_pk_kwd",
    "_pk_source",
    "_twitter_sess_id",
    "aff",
    "aff_id",
    "affiliate_id",
    "campaign",
    "campaign_id",
    "ceneo_spo",
    "cid",
    "cmpid",
    "correlation_id",
    "dclid",
    "epik",
    "fb_action_ids",
    "fb_action_types",
    "fb_ref",
    "fb_source",
    "fbclid",
    "from",
    "from_source",
    "gbraid",
    "gclid",
    "gclsrc",
    "gs_l",
    "hsCtaTracking",
    "hsa_acc",
    "hsa_ad",
    "hsa_cam",
    "hsa_grp",
    "hsa_kw",
    "hsa_mt",
    "hsa_net",
    "hsa_ol",
    "hsa_src",
    "hsa_ver",
    "icid",
    "igsh",
    "igshid",
    "itm_campaign",
    "itm_content",
    "itm_medium",
    "itm_source",
    "itm_term",
    "mc_cid",
    "mc_eid",
    "mc_tc",
    "mkt_tok",
    "ml_subscriber",
    "ml_subscriber_hash",
    "msclkid",
    "msi",
    "ncid",
    "oly_anon_id",
    "oly_enc_id",
    "os_ehash",
    "partner_id",
    "rb_clickid",
    "ref",
    "ref_src",
    "ref_url",
    "s_cid",
    "sc_campaign",
    "sc_channel",
    "sc_content",
    "sc_country",
    "sc_geo",
    "sc_medium",
    "sc_outcome",
    "share_id",
    "source",
    "sourceid",
    "spm",
    "spm_id",
    "srsltid",
    "tracking_source",
    "trk",
    "trkCampaign",
    "ttclid",
    "twclid",
    "utm_campaign",
    "utm_content",
    "utm_id",
    "utm_medium",
    "utm_reader",
    "utm_source",
    "utm_term",
    "vero_conv",
    "vero_id",
    "wbraid",
    "wickedid",
    "wtmc",
    "wtrid",
    "wtzmc",
    "yclid",
    "zanpid",
];

fn is_tracking_param(key: &str) -> bool {
    TRACKING_PARAMS.binary_search(&key).is_ok()
}

pub fn strip_tracking_params(url: &mut Url) {
    if url.query().is_none() {
        return;
    }

    let needs_cleaning = url.query_pairs().any(|(k, _)| is_tracking_param(&k));
    if !needs_cleaning {
        return;
    }

    let original_len = url.query().map_or(0, str::len);
    let kept: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(k, _)| !is_tracking_param(k))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    if kept.is_empty() {
        url.set_query(None);
        return;
    }

    let mut serializer = url::form_urlencoded::Serializer::new(String::with_capacity(original_len));
    for (k, v) in &kept {
        serializer.append_pair(k, v);
    }
    url.set_query(Some(&serializer.finish()));
}

pub fn strip_tracking(input: impl AsRef<str>) -> Result<String, url::ParseError> {
    let mut url = Url::parse(input.as_ref())?;
    strip_tracking_params(&mut url);
    Ok(url.into())
}

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
        .filter_map(|m| {
            let cleaned = strip_tracking(m.as_str()).ok()?;
            let url = clean_url(&cleaned);
            (!url.starts_with("discord.com/channels") && !url.starts_with("fmhy.net"))
                .then_some(url.to_owned())
        })
        .collect();

    Some(matches).filter(|m| !m.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_params_is_sorted() {
        assert!(
            TRACKING_PARAMS.windows(2).all(|w| w[0] < w[1]),
            "TRACKING_PARAMS must stay sorted for binary_search to work"
        );
    }

    #[test]
    fn strips_known_params_keeps_rest() {
        let mut url =
            Url::parse("https://example.com/page?utm_source=twitter&fbclid=123&good=keep").unwrap();
        strip_tracking_params(&mut url);
        assert_eq!(url.as_str(), "https://example.com/page?good=keep");
    }

    #[test]
    fn no_query_is_untouched() {
        let mut url = Url::parse("https://example.com/page").unwrap();
        strip_tracking_params(&mut url);
        assert_eq!(url.as_str(), "https://example.com/page");
    }

    #[test]
    fn all_tracking_params_removed_clears_query() {
        let mut url = Url::parse("https://example.com/page?utm_source=x&fbclid=y").unwrap();
        strip_tracking_params(&mut url);
        assert_eq!(url.as_str(), "https://example.com/page");
    }

    #[test]
    fn strip_tracking_accepts_str_and_string() {
        let a = strip_tracking("https://example.com/x?gclid=1").unwrap();
        let b = strip_tracking(String::from("https://example.com/x?gclid=1")).unwrap();
        assert_eq!(a, b);
        assert_eq!(a, "https://example.com/x");
    }
}

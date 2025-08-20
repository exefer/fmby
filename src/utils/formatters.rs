use entity::{sea_orm_active_enums::WikiUrlStatus, wiki_urls};

pub trait UrlFormatter {
    fn format_for_embed(&self, status: &WikiUrlStatus) -> Option<String>;
}

impl UrlFormatter for Vec<String> {
    fn format_for_embed(&self, _status: &WikiUrlStatus) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        Some(
            self.iter()
                .map(|url| format!("- {url}"))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

impl UrlFormatter for [String] {
    fn format_for_embed(&self, status: &WikiUrlStatus) -> Option<String> {
        self.to_vec().format_for_embed(status)
    }
}

impl UrlFormatter for Vec<wiki_urls::Model> {
    fn format_for_embed(&self, status: &WikiUrlStatus) -> Option<String> {
        let filtered: Vec<_> = self.iter().filter(|e| e.status == *status).collect();
        if filtered.is_empty() {
            return None;
        }

        let lines = match status {
            WikiUrlStatus::Pending | WikiUrlStatus::Removed => filtered
                .iter()
                .filter_map(
                    |entry| match (entry.guild_id, entry.channel_id, entry.message_id) {
                        (Some(guild_id), Some(channel_id), Some(message_id)) => Some(format!(
                            "- {} - https://discord.com/channels/{}/{}/{}",
                            entry.url, guild_id, channel_id, message_id
                        )),
                        _ => None,
                    },
                )
                .collect::<Vec<_>>(),
            _ => filtered
                .iter()
                .map(|entry| format!("- {}", entry.url))
                .collect::<Vec<_>>(),
        };

        Some(lines.join("\n"))
    }
}

impl UrlFormatter for [wiki_urls::Model] {
    fn format_for_embed(&self, status: &WikiUrlStatus) -> Option<String> {
        self.to_vec().format_for_embed(status)
    }
}

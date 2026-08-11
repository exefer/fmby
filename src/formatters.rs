use std::fmt::Write;

use crate::entities::enums::WikiUrlStatus;
use crate::entities::wiki_urls;

pub trait UrlFormatter {
    fn format_for_embed(&self, status: &WikiUrlStatus) -> Option<String>;
}

impl UrlFormatter for Vec<wiki_urls::Model> {
    fn format_for_embed(&self, status: &WikiUrlStatus) -> Option<String> {
        let mut lines = String::new();
        for entry in self.iter().filter(|e| e.status == *status) {
            match status {
                WikiUrlStatus::Pending | WikiUrlStatus::Removed => {
                    let (Some(guild_id), Some(channel_id), Some(message_id)) =
                        (entry.guild_id, entry.channel_id, entry.message_id)
                    else {
                        continue;
                    };
                    if !lines.is_empty() {
                        lines.push('\n');
                    }
                    let _ = write!(
                        lines,
                        "- {} - https://discord.com/channels/{guild_id}/{channel_id}/{message_id}",
                        entry.url
                    );
                }
                WikiUrlStatus::Added => {
                    if !lines.is_empty() {
                        lines.push('\n');
                    }
                    let _ = write!(lines, "- {}", entry.url);
                }
            }
        }
        (!lines.is_empty()).then_some(lines)
    }
}

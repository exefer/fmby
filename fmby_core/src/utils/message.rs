use poise::serenity_prelude::Message;

/// Gets message content, falling back to referenced message content if the main content is empty.
/// Returns `None` if both the message and its referenced message (if any) have empty content.
pub fn get_content_or_referenced(message: &Message) -> Option<&str> {
    if !message.content.is_empty() {
        Some(message.content.as_str())
    } else {
        message
            .referenced_message
            .as_ref()
            .filter(|m| !m.content.is_empty())
            .map(|m| m.content.as_str())
    }
}

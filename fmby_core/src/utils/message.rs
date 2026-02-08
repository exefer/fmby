use std::borrow::Cow;

use poise::serenity_prelude::{Http, Message, MessageReferenceKind};

/// Gets message content, falling back to referenced message content if the main content is empty.
/// Returns `None` if both the message and its referenced message (if any) have empty content.
pub async fn get_content_or_referenced<'a>(
    http: &'a Http,
    message: &'a Message,
) -> Option<Cow<'a, str>> {
    if !message.content.is_empty() {
        return Some(Cow::Borrowed(message.content.as_str()));
    }

    let msg_ref = message
        .message_reference
        .as_ref()
        .filter(|m| m.kind == MessageReferenceKind::Forward)?;

    let referenced = http
        .get_message(msg_ref.channel_id, msg_ref.message_id?)
        .await
        .ok()?;

    (!referenced.content.is_empty()).then(|| Cow::Owned(referenced.content.into_string()))
}

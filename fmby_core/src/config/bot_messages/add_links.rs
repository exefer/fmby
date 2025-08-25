#![allow(unused)]

use crate::generate_message_getters;

#[derive(Debug, Default)]
pub struct AddLinksMessages {
    pub message: Message,
}

#[derive(Debug)]
pub struct Message {
    pub already_added: &'static str,
    pub already_pending: &'static str,
    pub previously_removed: &'static str,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            already_added: "Link(s) already in the wiki:",
            already_pending: "Links(s) already in queue:",
            previously_removed: "Links(s) previously removed from the wiki:",
        }
    }
}

generate_message_getters!(AddLinksMessages,
    add_links.message.already_added => get_message_already_added,
    add_links.message.already_pending => get_message_already_pending,
    add_links.message.previously_removed => get_message_previously_removed,
);

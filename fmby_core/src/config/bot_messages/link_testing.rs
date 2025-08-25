#![allow(unused)]
use crate::generate_message_getters;

#[derive(Debug, Default)]
pub struct LinkTestingMessages {
    pub thread_create: ThreadCreate,
    pub thread_update: ThreadUpdate,
}

#[derive(Debug)]
pub struct ThreadCreate {
    pub welcome: &'static str,
}

#[derive(Debug)]
pub struct ThreadUpdate {
    pub rejected_closing: &'static str,
    pub approved_closing: &'static str,
    pub reopened: &'static str,
}

impl Default for ThreadCreate {
    fn default() -> Self {
        ThreadCreate {
            welcome: "Thread opened by {owner} - join in, share your thoughts, and keep the discussion going!",
        }
    }
}

impl Default for ThreadUpdate {
    fn default() -> Self {
        ThreadUpdate {
            rejected_closing: "{owner}: thread closed as rejected.",
            approved_closing: "{owner}: thread closed as approved; link(s) will be added to the wiki.",
            reopened: "{owner}: your previously rejected thread has been reopened; feel free to continue discussing and defending the link(s) you were testing.",
        }
    }
}

generate_message_getters!(LinkTestingMessages,
    link_testing.thread_create.welcome => get_thread_create_welcome [owner],
    link_testing.thread_update.rejected_closing => get_thread_update_rejected [owner],
    link_testing.thread_update.approved_closing => get_thread_update_approved [owner],
    link_testing.thread_update.reopened => get_thread_update_reopened [owner],
);

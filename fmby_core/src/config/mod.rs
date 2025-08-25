mod bot_messages;
pub use bot_messages::add_links::AddLinksMessages;
pub use bot_messages::link_testing::LinkTestingMessages;

#[derive(Debug, Default)]
pub struct BotMessages {
    pub link_testing: LinkTestingMessages,
    pub add_links: AddLinksMessages,
}

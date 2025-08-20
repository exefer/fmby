mod add_links;
mod link_testing;
pub use add_links::AddLinksMessages;
pub use link_testing::LinkTestingMessages;

#[derive(Debug, Default)]
pub struct MessagesConfig {
    pub link_testing: LinkTestingMessages,
    pub add_links: AddLinksMessages,
}

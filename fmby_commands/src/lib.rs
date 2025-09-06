pub mod fmby;
pub mod meta;
pub mod rss;
pub mod sql;
pub mod system;
pub use fmby_core::{
    error::Error,
    structs::{Command, Context, Data},
};

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    let commands: Vec<crate::Command> = meta::commands()
        .into_iter()
        .chain(fmby::commands())
        .chain(sql::commands())
        .chain(rss::commands())
        .chain(system::commands())
        .collect();

    commands
}

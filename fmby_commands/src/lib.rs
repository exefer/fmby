pub mod fmby;
pub mod fun;
pub mod meta;
pub mod rss;
pub mod sql;
pub mod system;
pub(crate) use fmby_core::{
    error::Error,
    structs::{Command, Context},
};

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    let commands: Vec<crate::Command> = meta::commands()
        .into_iter()
        .chain(fmby::commands())
        .chain(sql::commands())
        .chain(rss::commands())
        .chain(system::commands())
        .chain(fun::commands())
        .collect();

    commands
}

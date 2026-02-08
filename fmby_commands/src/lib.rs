pub mod fmby;
pub mod fun;
pub mod meta;
pub mod rss;
pub mod sql;

pub(crate) use fmby_core::error::Error;
pub(crate) use fmby_core::structs::{Command, Context};

pub fn commands() -> Vec<Command> {
    meta::commands()
        .into_iter()
        .chain(fmby::commands())
        .chain(sql::commands())
        .chain(rss::commands())
        .chain(fun::commands())
        .collect()
}

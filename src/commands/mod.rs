pub mod fmby;
pub mod fun;
pub mod meta;
pub mod rss;
pub mod sql;

use crate::error::Error;
use crate::types::{Command, Context};

pub fn commands() -> Vec<Command> {
    meta::commands()
        .into_iter()
        .chain(fmby::commands())
        .chain(sql::commands())
        .chain(rss::commands())
        .chain(fun::commands())
        .collect()
}

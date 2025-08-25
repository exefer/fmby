pub use fmby_core::{
    error::Error,
    structs::{Command, Context, Data},
};
pub mod fmby;
pub mod meta;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    let commands: Vec<crate::Command> = meta::commands()
        .into_iter()
        .chain(fmby::commands())
        .collect();

    commands
}

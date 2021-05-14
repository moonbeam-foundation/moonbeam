#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod command;

#[cfg(feature = "cli")]
pub use cli::*;

#[cfg(feature = "cli")]
pub use command::*;

#[cfg(feature = "cli")]
pub use sc_cli::{Error, Result};

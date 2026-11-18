use std::io::{self, Write};

use clap::ValueEnum;
use eros::Context;
use serde::Serialize;

use crate::error::Result;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

pub fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value).context("could not serialize JSON output")?;
    writeln!(stdout).context("could not write JSON output")?;
    Ok(())
}

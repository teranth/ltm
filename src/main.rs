mod commands;
mod db;
mod formatting;
mod json_formatting;
mod models;
mod validation;
mod interactive;
mod suggestions;
mod feedback;

use anyhow::Result;
use clap::Parser;
use commands::{Cli, CommandHandler};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = db::Database::new().await?;
    let mut handler = CommandHandler::new(db);
    handler.handle_command(cli).await
}

// src/cli.rs

mod args;
mod dto;
mod handlers;

use clap::Parser;

use crate::cli::args::{Cli, Commands};
use crate::cli::handlers::*;

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, name } => handle_init(path, name, cli.format),
        Commands::Inspect { path } => handle_inspect(path, cli.format),
        Commands::List { path } => handle_list(path, cli.format),
        Commands::AddEntry { path, title } => handle_add_entry(path, title, cli.format),
        Commands::AddField {
            path,
            entry,
            field_args,
        } => handle_add_field(path, entry, field_args, cli.format),
        Commands::View { path, entry } => handle_view(path, entry, cli.format),
        Commands::Dump { path } => handle_dump(path),
    }
}

use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short, long, default_value = ".")]
    pub repo_path: PathBuf,
}

#[derive(Subcommand)]
pub enum Command {
    Prompt,
    Branches,
}
mod cli;
mod primitives;
mod git;
mod display;

use crate::cli::{dir_status, dump_branches, get_prompt, Cli, Command};

use crate::primitives::FuError;
use clap::Parser;

fn main() -> Result<(), FuError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Prompt => get_prompt(&cli.repo_path),
        Command::Branches => dump_branches(&cli.repo_path),
        Command::DirStatus => dir_status(&cli.repo_path),
    }
}

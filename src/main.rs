mod cli;
mod core;

use crate::cli::{Cli, Command};
use crate::core::{dump_branches, gather_git_repo};
use crate::core::{get_repo_state, FuError};
use clap::Parser;

fn main() -> Result<(), FuError> {
    let cli = Cli::parse();
    let repo_result = gather_git_repo(cli.repo_path);
    if let Ok(repo) = repo_result {
        match cli.command {
            Command::Prompt => Ok(println!("{}", get_repo_state(&repo)?)),
            Command::Branches => dump_branches(&repo),
        }
    } else {
        Ok(())
    }
}

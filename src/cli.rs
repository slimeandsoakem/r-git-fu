use crate::display::{console_dump};
use crate::git::{gather_git_repo, get_branch_info, get_multi_directory_status, get_repo_state, print_repo_table};
use crate::primitives::FuError;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    DirStatus,
}

pub fn get_prompt(path: &PathBuf) -> Result<(), FuError> {
    let repo_result = gather_git_repo(path);
    if let Ok(repo) = repo_result {
        Ok(println!("{}", get_repo_state(&repo)?))
    } else {
        Ok(())
    }
}

pub fn dump_branches(path: &PathBuf) -> Result<(), FuError> {
    let repo_result = gather_git_repo(path);
    if let Ok(repo) = repo_result {
        let branch_info = get_branch_info(&repo)?;
        console_dump(branch_info);
        Ok(())
    } else {
        Ok(())
    }
}

pub fn dir_status(path: &PathBuf) -> Result<(), FuError> {
    let full_results = get_multi_directory_status(path)?;
    print_repo_table(full_results);
    Ok(())
}



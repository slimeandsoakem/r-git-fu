
use crate::git::{gather_git_repo, get_branch_info, get_multi_directory_status, get_repo_state, print_branch_table, print_repo_table};
use crate::primitives::{FuError};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short = 'd', long, default_value = ".")]
    pub repo_path: PathBuf,
    #[arg(short, long, default_value = "false")]
    pub fetch: bool,
    #[arg(short, long, default_value = "2500")]
    pub timeout: u64,
    #[arg(long, short, default_value = "false")]
    pub remote_status: bool,
    #[arg(long, short, default_value = "false")]
    pub plain_tables: bool,
}

#[derive(Subcommand)]
pub enum Command {
    Prompt,
    Branches,
    DirStatus,
}


pub fn get_prompt(path: &PathBuf, remote_status: bool) -> Result<(), FuError> {
    let repo_result = gather_git_repo(path);
    if let Ok(repo) = repo_result {
        Ok(println!("{}", get_repo_state(&repo, false, remote_status, 0)?))
    } else {
        Ok(())
    }
}

pub fn dump_branches(path: &PathBuf, plain_tables: bool) -> Result<(), FuError> {
    let repo_result = gather_git_repo(path);
    if let Ok(repo) = repo_result {
        let branch_info = get_branch_info(&repo)?;
        if let Some(branch_summary) = branch_info {
            print_branch_table(branch_summary, plain_tables)
        }
        Ok(())
    } else {
        Ok(())
    }
}

pub fn dir_status(path: &PathBuf, fetch: bool, timeout_ms: u64, plain_tables: bool) -> Result<(), FuError> {
    let full_results = get_multi_directory_status(path, fetch, timeout_ms)?;
    print_repo_table(full_results, plain_tables);
    Ok(())
}



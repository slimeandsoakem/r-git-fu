use chrono::{DateTime, TimeZone, Utc};
use git2::BranchType;
use git2::{Error as Git2Error, Reference, Repository};
use owo_colors::OwoColorize;
use std::env::VarError;
use std::fmt::Display;
use std::io::{self, Write};
use std::path::PathBuf;
use thiserror::Error as ThisError;

#[derive(Debug)]
pub struct RepoStatus {
    pub branch: BranchState,
    pub dirty: DirtyState,
    pub position: Option<Position>,
    pub head_oid: git2::Oid,
}

impl Display for RepoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let branch_str = match &self.branch {
            BranchState::Named(name) => name.clone().magenta().to_string(),
            BranchState::Detached => format!("{}", &self.head_oid.to_string()[..7])
                .cyan()
                .to_string(),
        };

        // Position symbols
        let mut pos = String::new();
        if let Some(position) = &self.position {
            if position.ahead > 0 {
                pos.push_str(&format!("↑{}", position.ahead));
            }
            if position.behind > 0 {
                if !pos.is_empty() {
                    pos.push(' ');
                }
                pos.push_str(&format!("↓{}", position.behind));
            }
        }

        let mut dirty = String::new();
        if self.dirty.index > 0 {
            dirty.push_str(&format!("●{}", self.dirty.index).red().to_string());
        }
        if self.dirty.worktree > 0 {
            dirty.push_str(&format!("+{}", self.dirty.worktree).blue().to_string());
        }

        if self.dirty.index == 0 && self.dirty.worktree == 0 {
            dirty.push_str(&'✔'.green().to_string()); // green tick
        }

        // Combine
        let mut parts = vec![branch_str];
        if !pos.is_empty() || !dirty.is_empty() {
            parts.push(format!("{}|{}", pos, dirty));
        }

        write!(f, "{}", parts.join(""))
    }
}

#[derive(Debug)]
pub struct Position {
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug)]
pub enum BranchState {
    Named(String),
    Detached,
}

#[derive(Debug)]
pub struct DirtyState {
    pub worktree: usize, // number of uncommitted changes in worktree
    pub index: usize,    // number of staged changes
}

#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub commit_time: i64,
    pub iso_date: String,
    pub delta: String,
}
impl Display for BranchInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!(
            "{} {} {}",
            &self.iso_date.green(),
            &self.delta.blue(),
            &self.name.white()
        )
        .fmt(f)
    }
}

#[derive(ThisError, Debug)]
pub enum FuError {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    Git2Error(#[from] Git2Error),

    #[error(transparent)]
    VarError(#[from] VarError),
}

fn safe_println(s: &str) {
    if let Err(e) = writeln!(io::stdout(), "{}", s) {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            // Only panic for other IO errors
            panic!("stdout error: {}", e);
        }
        // Broken pipe: exit silently
        std::process::exit(0);
    }
}

pub fn gather_git_repo(path_buf: PathBuf) -> Result<Repository, FuError> {
    let git_dir = path_buf.join(".git");

    if !git_dir.exists() || !git_dir.is_dir() {
        return Err(FuError::Custom(format!(
            "No .git directory found at {}",
            path_buf.display()
        )));
    }

    let repo = git2::Repository::discover(path_buf)?;
    Ok(repo)
}

fn timestamp_to_datetime(ts: i64) -> Result<DateTime<Utc>, FuError> {
    let timestamp = Utc
        .timestamp_opt(ts, 0)
        .single()
        .ok_or(FuError::Custom("Time out of range".to_string()))?;
    Ok(timestamp)
}
fn format_commit_time(ts: i64) -> Result<(String, String), FuError> {
    let datetime = timestamp_to_datetime(ts)?;
    let iso_date = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
    let delta = format!(
        "{}",
        humantime::format_duration(std::time::Duration::from_secs(
            (Utc::now().timestamp() - ts) as u64
        ))
    );
    Ok((iso_date, delta))
}
pub fn get_branch_info(repo: &Repository) -> Result<Option<Vec<BranchInfo>>, FuError> {
    let mut branches = Vec::new();
    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        let name = branch.name()?.unwrap().to_string();

        let commit = branch.get().peel_to_commit()?;
        let (iso_date, delta) = format_commit_time(commit.time().seconds())?;

        branches.push(BranchInfo {
            name,
            commit_time: commit.time().seconds(),
            iso_date,
            delta,
        });
        branches.sort_by(|a, b| b.commit_time.cmp(&a.commit_time));
    }
    if branches.is_empty() {
        Ok(None)
    } else {
        Ok(Some(branches))
    }
}

pub fn console_dump<T>(outbound_array: Option<Vec<T>>)
where
    T: Display,
{
    if let Some(vec) = outbound_array {
        for x in vec {
            safe_println(&format!("{}", x));
        }
    }
}

fn get_position(head_ref: &Reference, repo: &Repository) -> Result<Option<Position>, FuError> {
    // Detached HEAD → skip
    if !head_ref.is_branch() {
        return Ok(None);
    }

    let branch = repo.find_branch(head_ref.shorthand().unwrap(), BranchType::Local)?;

    let upstream = match branch.upstream() {
        Ok(u) => u,
        Err(_) => return Ok(None), // no upstream configured
    };

    let local_oid = branch.into_reference().target().unwrap();
    let upstream_oid = upstream.into_reference().target().unwrap();

    let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
    Ok(Some(Position { ahead, behind }))
}

fn get_branch_state(head_ref: &Reference) -> Result<BranchState, FuError> {
    let branch = if head_ref.is_branch() {
        BranchState::Named(
            head_ref
                .shorthand()
                .ok_or(FuError::Custom("No name for a named branch".to_string()))?
                .to_string(),
        )
    } else {
        BranchState::Detached
    };
    Ok(branch)
}

fn get_dirty(repo: &Repository) -> Result<DirtyState, FuError> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut worktree_dirty = 0;
    let mut index_dirty = 0;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.is_wt_modified() || s.is_wt_new() || s.is_wt_deleted() {
            worktree_dirty += 1;
        }
        if s.is_index_modified() || s.is_index_new() || s.is_index_deleted() {
            index_dirty += 1;
        }
    }

    let dirty = DirtyState {
        worktree: worktree_dirty,
        index: index_dirty,
    };
    Ok(dirty)
}

pub fn get_repo_state(repo: &Repository) -> Result<RepoStatus, FuError> {
    let head = repo.head()?;
    let head_oid = head.target().unwrap();
    let branch = get_branch_state(&head)?;
    let dirty = get_dirty(&repo)?;
    let position = get_position(&head, &repo)?;
    Ok(RepoStatus {
        branch,
        dirty,
        position,
        head_oid,
    })
}

pub fn dump_branches(repo: &Repository) -> Result<(), FuError> {
    let branch_info = get_branch_info(&repo)?;
    console_dump(branch_info);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn full_commit_history(repo: &Repository) -> Result<(), FuError> {
        let mut reverse_walk = repo.revwalk()?;
        reverse_walk.push_head()?;
        reverse_walk.set_sorting(git2::Sort::TIME)?;

        for oid in reverse_walk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;

            println!(
                "{} {} {}",
                format_commit_time(commit.time().seconds())?.0,
                &commit.id().to_string()[..7],
                commit.summary().unwrap_or("")
            );
        }
        Ok(())
    }

    #[test]
    fn test_gather_git_status() -> Result<(), FuError> {
        let test_repo = PathBuf::from(std::env::var("FU_TEST_REPO")?.to_string());
        let repo = gather_git_repo(test_repo)?;
        full_commit_history(&repo)?;
        dump_branches(&repo)?;

        let repo_state = get_repo_state(&repo)?;
        println!("{}", repo_state);

        Ok(())
    }
}

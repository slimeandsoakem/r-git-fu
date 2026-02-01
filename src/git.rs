use crate::display::standard_table_setup;
use crate::primitives::{
    BranchInfo, BranchState, DirtyState, FuError, Position, RemoteStatus, RepoStatus,
};
use comfy_table::{Cell, Color};
use git2::{BranchType, Oid, Reference, Repository};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const ORIGIN: &str = "origin";

pub fn gather_git_repo(path_buf: &PathBuf) -> Result<Repository, FuError> {
    let git_dir = path_buf.join(".git");

    if !git_dir.exists() || !git_dir.is_dir() {
        return Err(FuError::Custom(format!(
            "No .git directory found at {}",
            path_buf.display()
        )));
    }

    let repo = Repository::discover(path_buf)?;
    Ok(repo)
}

pub fn get_branch_info(repo: &Repository) -> Result<Option<Vec<BranchInfo>>, FuError> {
    let mut branches = Vec::new();
    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        let name = branch.name()?.unwrap().to_string();

        let commit = branch.get().peel_to_commit()?;
        let (iso_date, delta) = crate::display::format_commit_time(commit.time().seconds())?;

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

pub fn get_position(head_ref: &Reference, repo: &Repository) -> Result<Option<Position>, FuError> {
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

pub fn get_branch_state(head_ref: &Reference) -> Result<BranchState, FuError> {
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

pub fn get_dirty(repo: &Repository) -> Result<DirtyState, FuError> {
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

fn fetch_git_with_timeout(repo_path: &str, remote: &str, timeout_ms: u64) -> Result<bool, FuError> {
    let mut child = Command::new("git")
        .args(["-C", repo_path, "fetch", "--prune", "--quiet", remote])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let timeout = Duration::from_millis(timeout_ms);

    match child.wait_timeout(timeout)? {
        Some(_status) => Ok(true),
        None => {
            // timed out → kill process
            let _ = child.kill();
            let _ = child.wait();
            Ok(false)
        }
    }
}

fn get_remote_status(
    fetch: bool,
    repo: &Repository,
    head: &Reference,
    head_oid: &Oid,
    timeout_ms: u64,
) -> Result<Option<RemoteStatus>, FuError> {
    let work_dir = &repo
        .workdir()
        .ok_or(FuError::Custom("Cannot find workdir".to_string()))?
        .to_str()
        .ok_or(FuError::Custom(
            "Cannot convert workdir to string".to_string(),
        ))?;

    if !head.is_branch() {
        return Ok(None);
    }

    let mut refreshed: bool = false;

    if fetch {
        refreshed = fetch_git_with_timeout(work_dir, ORIGIN, timeout_ms)?;
    }

    let branch_name = head
        .shorthand()
        .ok_or(FuError::Custom("No branch name".to_string()))?;
    let remote_ref = format!("refs/remotes/{}/{}", ORIGIN, branch_name);
    let remote_oid = match repo.refname_to_id(&remote_ref) {
        Ok(oid) => oid,
        Err(_) => return Ok(None), // upstream not found
    };

    let (ahead, behind) = repo.graph_ahead_behind(*head_oid, remote_oid)?;
    let position = Position { ahead, behind };
    let remote_status = RemoteStatus {
        position: Some(position),
        refreshed,
    };

    Ok(Some(remote_status))
}

pub fn get_repo_state(
    repo: &Repository,
    fetch: bool,
    remote_status: bool,
    timeout_ms: u64,
) -> Result<RepoStatus, FuError> {
    let head = repo.head()?;
    let head_oid = head.target().unwrap();
    let branch = get_branch_state(&head)?;
    let dirty = get_dirty(&repo)?;
    let position = get_position(&head, &repo)?;
    let remote_status = if remote_status {
        get_remote_status(fetch, &repo, &head, &head_oid, timeout_ms)?
    } else {
        None
    };
    Ok(RepoStatus {
        branch,
        dirty,
        position,
        head_oid,
        remote_status,
    })
}

pub fn get_multi_directory_status(
    path_buf: &PathBuf,
    fetch: bool,
    timeout_ms: u64,
) -> Result<Option<HashMap<String, RepoStatus>>, FuError> {
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(path_buf)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }

    let mut current_fetch_status: bool = fetch;

    let mut status_results: HashMap<String, RepoStatus> = HashMap::new();
    for dir in dirs {
        let repo_result = gather_git_repo(&dir);
        let name_osstr = dir
            .file_name()
            .ok_or(FuError::Custom("Cannot determine name".to_string()))?;
        let name = name_osstr.to_string_lossy().to_string();

        if let Ok(repo) = repo_result {
            let repo_status_result = get_repo_state(&repo, current_fetch_status, true, timeout_ms);
            if let Ok(repo_status) = repo_status_result {
                current_fetch_status = repo_status
                    .remote_status
                    .as_ref()
                    .map(|remote_status| remote_status.refreshed)
                    .unwrap_or(true)
                    && current_fetch_status;
                status_results.insert(name, repo_status);
            } else {
                status_results.insert(name, RepoStatus::broken_state("broken-head".to_string()));
            }
        }
    }
    if status_results.is_empty() {
        Ok(None)
    } else {
        Ok(Some(status_results))
    }
}

pub fn print_repo_table(result_option: Option<HashMap<String, RepoStatus>>, plain_tables: bool) {
    if let Some(results) = result_option {
        let mut rows: Vec<_> = results.into_iter().collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        let mut table = standard_table_setup(plain_tables);
        table.set_header(vec![
            Cell::new("Repo"),
            Cell::new("Branch"),
            Cell::new("Dirty"),
            Cell::new("Position"),
            Cell::new("Remote"),
        ]);

        for (name, status) in rows {
            let dirty_val = if status.dirty.worktree + status.dirty.index == 0 {
                "".to_string()
            } else {
                format!("●{}+{}", status.dirty.worktree, status.dirty.index)
            };

            let dirty_cell = if dirty_val.is_empty() {
                Cell::new("").fg(Color::Red)
            } else {
                Cell::new(&dirty_val).fg(Color::Red)
            };

            let position_val = match &status.position {
                Some(pos) if pos.ahead > 0 || pos.behind > 0 => {
                    format!("↑{}↓{}", pos.ahead, pos.behind)
                }
                _ => "".to_string(),
            };

            let position_cell = if position_val.is_empty() {
                Cell::new("").fg(Color::Green)
            } else {
                Cell::new(&position_val).fg(Color::Green)
            };

            let remote_cell = match &status.remote_status {
                Some(remote_position) => {
                    let string_legend = match &remote_position.position {
                        Some(pos) if pos.ahead > 0 || pos.behind > 0 => {
                            format!("↑{}↓{}", pos.ahead, pos.behind)
                        }
                        _ => "".to_string(),
                    };
                    if remote_position.refreshed {
                        Cell::new(&string_legend).fg(Color::Green)
                    } else {
                        Cell::new(string_legend).fg(Color::Yellow)
                    }
                }
                _ => Cell::new("").fg(Color::Green),
            };

            let (name_cell, branch_cell) = match (
                dirty_val.is_empty(),
                position_val.is_empty(),
                status.head_oid.is_zero(),
            ) {
                (true, true, false) => (
                    Cell::new(name).fg(Color::White),
                    Cell::new(&status.branch_name(false)).fg(Color::White),
                ),
                (true, true, true) => (
                    Cell::new(name).fg(Color::Magenta),
                    Cell::new(&status.branch_name(false)).fg(Color::Magenta),
                ),
                (true, _, _) | (_, true, _) => (
                    Cell::new(name).fg(Color::Yellow),
                    Cell::new(&status.branch_name(false)).fg(Color::Yellow),
                ),
                _ => (
                    Cell::new(name).fg(Color::White),
                    Cell::new(&status.branch_name(false)).fg(Color::White),
                ),
            };

            table.add_row(vec![
                name_cell,
                branch_cell,
                dirty_cell,
                position_cell,
                remote_cell,
            ]);
        }

        println!("{}", table);
    }
}

pub fn print_branch_table(branch_summary: Vec<BranchInfo>, plain_tables: bool) {
    let mut table = standard_table_setup(plain_tables);
    table.set_header(vec![
        Cell::new("Last commit"),
        Cell::new("Age"),
        Cell::new("Branch name"),
    ]);

    for branch_info in branch_summary {
        
        table.add_row(vec![
            Cell::new(branch_info.iso_date).fg(Color::Green),
            Cell::new(branch_info.delta).fg(Color::Blue),
            Cell::new(branch_info.name).fg(Color::White),
        ]);
    }

    println!("{}", table);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{dump_branches, get_prompt};
    use crate::display::format_commit_time;

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
    fn test_gather_git_status_no_fetch() -> Result<(), FuError> {
        let test_repo = PathBuf::from(std::env::var("FU_TEST_REPO")?.to_string());
        let repo = gather_git_repo(&test_repo)?;
        full_commit_history(&repo)?;
        dump_branches(&test_repo, false)?;
        get_prompt(&test_repo, false)?;

        let repo_state = get_repo_state(&repo, false, false, 0)?;
        println!("{}", repo_state);

        Ok(())
    }

    #[test]
    fn test_gather_git_status_with_fetch() -> Result<(), FuError> {
        let test_repo = PathBuf::from(std::env::var("FU_TEST_REPO")?.to_string());
        let repo = gather_git_repo(&test_repo)?;
        let repo_state = get_repo_state(&repo, true, true, 2500)?;
        println!("{}", repo_state);

        Ok(())
    }

    #[test]
    fn test_tables() -> Result<(), FuError> {
        let test_state_row = RepoStatus {
            branch: BranchState::Named("test".to_string()),
            dirty: DirtyState {
                worktree: 1,
                index: 2,
            },
            position: Some(Position {
                ahead: 2,
                behind: 3,
            }),
            head_oid: Oid::zero(),
            remote_status: None,
        };
        let mut sample_output: HashMap<String, RepoStatus> = HashMap::new();
        sample_output.insert("long_name_to_test".to_string(), test_state_row);
        print_repo_table(Some(sample_output), false);

        Ok(())
    }
}

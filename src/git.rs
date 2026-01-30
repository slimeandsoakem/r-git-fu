use crate::primitives::{BranchInfo, BranchState, DirtyState, FuError, Position, RepoStatus};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::{ASCII_BORDERS_ONLY_CONDENSED};
use comfy_table::{Cell, Color, Table};
use git2::{BranchType, Reference, Repository};
use std::collections::HashMap;
use std::path::PathBuf;

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

pub fn get_multi_directory_status(
    path_buf: &PathBuf,
) -> Result<Option<HashMap<String, RepoStatus>>, FuError> {
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(path_buf)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }

    let mut status_results: HashMap<String, RepoStatus> = HashMap::new();
    for dir in dirs {
        let repo_result = gather_git_repo(&dir);
        if let Ok(repo) = repo_result {
            let repo_status = get_repo_state(&repo)?;
            if let Some(name_osstr) = dir.file_name() {
                let name = name_osstr.to_string_lossy().to_string();
                status_results.insert(name, repo_status);
            }
        }
    }
    if status_results.is_empty() {
        Ok(None)
    } else {
        Ok(Some(status_results))
    }
}

pub fn print_repo_table(result_option: Option<HashMap<String, RepoStatus>>) {
    if let Some(results) = result_option {
        let mut table = Table::new();
        table
            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
            .apply_modifier(UTF8_ROUND_CORNERS);
        table.load_preset(ASCII_BORDERS_ONLY_CONDENSED).set_header(vec![
            Cell::new("Repo"),
            Cell::new("Branch"),
            Cell::new("Dirty"),
            Cell::new("Position"),
        ]);

        for (name, status) in results {
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
                    format!("↑{} ↓{}", pos.ahead, pos.behind)
                }
                _ => "".to_string(),
            };

            let position_cell = if position_val.is_empty() {
                Cell::new("").fg(Color::Green)
            } else {
                Cell::new(&position_val).fg(Color::Green)
            };



            let (name_cell, branch_cell) = if dirty_val.is_empty() && position_val.is_empty() {
                (Cell::new(name).fg(Color::White), Cell::new(&status.branch_name(false)).fg(Color::White))
            } else {
                (Cell::new(name).fg(Color::Yellow), Cell::new(&status.branch_name(false)).fg(Color::Yellow))
            };

            table.add_row(vec![
                name_cell,
                branch_cell,
                dirty_cell,
                position_cell,
            ]);
        }

        println!("{}", table);
    }
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
    fn test_gather_git_status() -> Result<(), FuError> {
        let test_repo = PathBuf::from(std::env::var("FU_TEST_REPO")?.to_string());
        let repo = gather_git_repo(&test_repo)?;
        full_commit_history(&repo)?;
        dump_branches(&test_repo)?;
        get_prompt(&test_repo)?;

        let repo_state = get_repo_state(&repo)?;
        println!("{}", repo_state);

        let full_results = get_multi_directory_status(&PathBuf::from(
            "/Users/Simon/Documents/dataoperations/projects/bics/code/mbtp",
        ))?;
        print_repo_table(full_results);
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
            head_oid: git2::Oid::zero(),
        };
        let mut sample_output: HashMap<String, RepoStatus> = HashMap::new();
        sample_output.insert("long_name_to_test".to_string(), test_state_row);
        print_repo_table(Some(sample_output));

        Ok(())
    }
}

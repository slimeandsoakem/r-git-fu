use git2::Error as Git2Error;
use owo_colors::OwoColorize;
use std::env::VarError;
use std::fmt::Display;

use std::io::Error as IoError;
use thiserror::Error as ThisError;

#[derive(Debug)]
pub struct RemoteStatus {
    pub position: Option<Position>,
    pub refreshed: bool,
}

#[derive(Debug)]
pub struct RepoStatus {
    pub branch: BranchState,
    pub dirty: DirtyState,
    pub position: Option<Position>,
    pub head_oid: git2::Oid,
    pub remote_status: Option<RemoteStatus>,
}

impl RepoStatus {
    pub fn broken_state(broken_state: String) -> Self {
        RepoStatus {
            branch: BranchState::Named(broken_state),
            dirty: DirtyState {worktree:0, index:0},
            position: None,
            head_oid: git2::Oid::zero(),
            remote_status: None,
        }
    }

    pub fn branch_name(&self, colour_flag: bool) -> String {
        let mut branch_str = match &self.branch {
            BranchState::Named(name) => name.clone().to_string(),
            BranchState::Detached => format!("{}", &self.head_oid.to_string()[..7])
                .to_string(),
        };
        if colour_flag {
            match &self.branch {
                BranchState::Named(_name) => branch_str = branch_str.magenta().to_string(),
                BranchState::Detached => branch_str = branch_str.cyan().to_string(),
            };
        }
        branch_str
    }

    pub fn position_marker(&self) -> String {
        match &self.position {
            Some(pos) => {
                let mut s = String::new();
                let (ahead, behind) = pos.string_markers();
                if pos.ahead > 0 {
                    s.push_str(&ahead.green().to_string());
                }
                if pos.behind > 0 {
                    if !s.is_empty() {
                        s.push(' ');
                    }
                    s.push_str(&behind.red().to_string());
                }
                match &self.remote_status {
                    Some(remote_status) => {
                        if let Some(remote_position) = &remote_status.position {
                            let (remote_ahead, remote_behind) = remote_position.string_markers();
                            if remote_position.behind > 0 || remote_position.ahead > 0 {
                                let remote_string = format!("[{}|{}]", remote_ahead, remote_behind);
                                s.push_str(&remote_string.yellow().to_string());
                            }
                        }
                    }
                    None => {}
                }
                s
            }
            None => "".into(),
        }
    }

    pub fn dirty_marker(&self) -> String {
        if self.dirty.worktree == 0 && self.dirty.index == 0 {
            return "✔".green().to_string();
        }

        let mut s = String::new();

        s.push_str(&"●".red().to_string());

        if self.dirty.worktree > 0 {
            s.push_str(&format!("{}", self.dirty.worktree).yellow().to_string());
        }

        if self.dirty.index > 0 {
            s.push_str(&format!("+{}", self.dirty.index).yellow().to_string());
        }

        s
    }
}

impl Display for RepoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let branch_str = self.branch_name(true);
        let position_str = self.position_marker();
        let dirty = self.dirty_marker();

        let mut parts: Vec<String> = vec![branch_str];
        if !position_str.is_empty() || !dirty.is_empty() {
            parts.push(format!("{}|{}", position_str, dirty));
        }

        write!(f, "({})", parts.join(""))
    }
}

#[derive(Debug)]
pub struct Position {
    pub ahead: usize,
    pub behind: usize,
}

impl Position {
    pub fn string_markers(&self) -> (String, String) {
        let (mut ahead, mut behind) = (String::new(), String::new());
        if self.ahead > 0 {
            ahead.push_str(&format!("↑{}", self.ahead));
        }
        if self.behind > 0 {
            behind.push_str(&format!("↓{}", self.behind));
        }
        (ahead, behind)
    }
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

    #[error(transparent)]
    IoError(#[from] IoError),
}

//! State files for the Claude sessions that run in this server's panes without one. Claude Code lists
//! each running session in `<claude dir>/sessions/<pid>.json`, so the sidebar can show a session whose
//! `SessionStart` hook never wrote a record. Claude Code drops a hook that misses its timeout, which
//! can happen when many sessions start at once after a reboot.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{Record, Status, store, valid_session_id};
use crate::context::{self, Context};
use crate::process;
use crate::tmux::pane::{Pane, PaneId};

const SESSIONS_DIR: &str = "sessions";
const PROJECTS_DIR: &str = "projects";

/// What Claude Code writes about one of its running processes.
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Known {
    #[serde(default)]
    pub pid: u32,
    #[serde(default, rename = "sessionId")]
    pub session_id: String,
    #[serde(default)]
    pub cwd: String,
    /// When the process started, in milliseconds since the Unix epoch.
    #[serde(default, rename = "startedAt")]
    pub started_at: u64,
    /// `session:@window.%pane`; empty outside tmux.
    #[serde(default)]
    pub tmux: String,
}

impl Known {
    pub fn parse(json: &[u8]) -> Option<Self> {
        serde_json::from_slice(json)
            .ok()
            .filter(|k: &Self| valid_session_id(&k.session_id))
    }

    /// The pane Claude says it runs in.
    pub fn pane(&self) -> Option<PaneId> {
        self.tmux.rsplit('.').next().and_then(PaneId::parse)
    }
}

/// Every session Claude Code lists in its directory.
pub fn known(claude_dir: &Path) -> Vec<Known> {
    fs::read_dir(claude_dir.join(SESSIONS_DIR))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .filter_map(|p| fs::read(p).ok())
        .filter_map(|bytes| Known::parse(&bytes))
        .collect()
}

/// Where Claude Code keeps a session's transcript: the directory is the cwd with every character
/// other than an ASCII letter or digit turned into `-`.
pub fn transcript(claude_dir: &Path, cwd: &str, session_id: &str) -> PathBuf {
    let project: String = cwd
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    claude_dir
        .join(PROJECTS_DIR)
        .join(project)
        .join(format!("{session_id}.jsonl"))
}

/// The record a session starts with when no hook wrote one: idle since Claude started, like after
/// `SessionStart`. The transcript path is set only when the file is there.
fn record(known: &Known, pane: PaneId, ctx: &Context, now: u64) -> Record {
    let transcript = transcript(&ctx.claude_dir, &known.cwd, &known.session_id);
    let started = known.started_at / 1000;
    Record {
        session_id: known.session_id.clone(),
        state: Status::Idle,
        cwd: known.cwd.clone(),
        pane: pane.to_string(),
        socket: ctx.socket.clone(),
        pid: Some(known.pid),
        since: Some(if started > 0 { started } else { now }),
        updated_at: now,
        transcript: existing(&transcript),
        ..Record::default()
    }
}

/// The path as the record stores it, or empty when there is no such file.
fn existing(path: &Path) -> String {
    if path.exists() {
        path.to_string_lossy().into_owned()
    } else {
        String::new()
    }
}

/// The sessions the sidebar has already looked at. Each one costs a process check, so it is done once.
#[derive(Debug, Default)]
pub struct Seeded(HashSet<(u32, String)>);

impl Seeded {
    /// Writes a state file for every known session that runs in one of `panes` and has none yet.
    #[must_use]
    pub fn seed(self, ctx: &Context, panes: &[Pane]) -> Self {
        let now = context::now();
        let fresh: Vec<(Known, &Pane)> = known(&ctx.claude_dir)
            .into_iter()
            .filter(|k| !self.0.contains(&(k.pid, k.session_id.clone())))
            .filter(|k| !store::path(&ctx.state_dir, &k.session_id).exists())
            .filter_map(|k| {
                let pane = k.pane().and_then(|id| panes.iter().find(|p| p.id == id))?;
                Some((k, pane))
            })
            .collect();
        fresh
            .iter()
            // The pane id alone is not enough: another tmux server may use the same ids.
            .filter(|(k, pane)| process::alive(k.pid) && process::runs_under(k.pid, pane.pid))
            .for_each(|(k, pane)| {
                let _ = store::write_new(&ctx.state_dir, &record(k, pane.id, ctx, now));
            });
        let seen = fresh.into_iter().map(|(k, _)| (k.pid, k.session_id));
        Self(self.0.into_iter().chain(seen).collect())
    }
}

#[cfg(test)]
mod tests;

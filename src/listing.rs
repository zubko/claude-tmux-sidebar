//! Claude sessions and the windows without Claude, grouped by tmux session, in tmux's own session /
//! window / pane order.

use std::borrow::Cow;
use std::collections::HashSet;

use itertools::Itertools;

use crate::session::live::Claude;
use crate::tmux::pane::{self, Pane};

/// A group heading: a tmux session, or the sessions running outside tmux.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GroupName {
    Session(String),
    Outside,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub name: GroupName,
    pub claudes: Vec<Claude>,
    /// The pane the user works in, for each window of the session that runs no Claude.
    pub windows: Vec<Pane>,
}

fn group_name(claude: &Claude) -> GroupName {
    claude.pane.as_ref().map_or(GroupName::Outside, |p| {
        GroupName::Session(p.session.clone())
    })
}

/// tmux order; rows outside tmux have no pane, so they go by the folder they show.
fn order(claude: &Claude) -> (u32, u32, Cow<'_, str>, &str) {
    let (window, pane) = claude
        .pane
        .as_ref()
        .map_or((0, 0), |p| (p.window_index, p.pane_index));
    (window, pane, claude.folder(), &claude.record.session_id)
}

/// Every tmux session (with or without Claude) in tmux's order, then the sessions outside tmux.
pub fn groups(sessions: &[String], claudes: &[Claude], panes: &[Pane]) -> Vec<Group> {
    let windows = plain_windows(claudes, panes);
    let names = claudes.iter().map(group_name);
    sessions
        .iter()
        .cloned()
        .map(GroupName::Session)
        .chain(names.clone().filter(|n| *n != GroupName::Outside))
        .chain(names.filter(|n| *n == GroupName::Outside))
        .unique()
        .map(|name| Group {
            claudes: claudes
                .iter()
                .filter(|c| group_name(c) == name)
                .cloned()
                .sorted_by(|a, b| order(a).cmp(&order(b)))
                .collect(),
            windows: windows
                .iter()
                .filter(|p| GroupName::Session(p.session.clone()) == name)
                .cloned()
                .collect(),
            name,
        })
        .collect()
}

/// The pane the user works in, for each window without Claude, in window order. `list-panes -a` repeats
/// a linked window under every session it is in; its last copy is kept, as `session::live` keeps the
/// last pane per id for a Claude in such a window.
fn plain_windows(claudes: &[Claude], panes: &[Pane]) -> Vec<Pane> {
    let busy: HashSet<_> = claudes
        .iter()
        .filter_map(|c| c.pane.as_ref())
        .map(|p| p.window_id)
        .collect();
    let last_first: Vec<Pane> = panes.iter().rev().cloned().collect();
    panes
        .iter()
        .map(|p| p.window_id)
        .unique()
        .filter(|w| !busy.contains(w))
        .filter_map(|w| pane::focused(&last_first, w))
        .cloned()
        .sorted_by_key(|p| p.window_index)
        .collect()
}

#[cfg(test)]
mod tests;

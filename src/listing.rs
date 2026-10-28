//! Claude sessions and the windows without Claude, grouped by tmux session, in tmux's own session /
//! window / pane order.

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

fn position(claude: &Claude) -> (u32, u32) {
    claude
        .pane
        .as_ref()
        .map_or((0, 0), |p| (p.window_index, p.pane_index))
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
                .sorted_by_key(position)
                .collect(),
            windows: windows
                .iter()
                .filter(|p| GroupName::Session(p.session.clone()) == name)
                .map(|&p| p.clone())
                .collect(),
            name,
        })
        .collect()
}

/// The pane the user works in, for each window without Claude, in window order.
fn plain_windows<'a>(claudes: &[Claude], panes: &'a [Pane]) -> Vec<&'a Pane> {
    let busy: HashSet<_> = claudes
        .iter()
        .filter_map(|c| c.pane.as_ref())
        .map(|p| p.window_id)
        .collect();
    panes
        .iter()
        .filter(|p| !busy.contains(&p.window_id))
        .unique_by(|p| p.window_id)
        .filter_map(|p| pane::focused(panes, p.window_id))
        .sorted_by_key(|p| p.window_index)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Record;
    use crate::tmux::pane::{Pane, fixture::pane};

    fn claude(id: &str, pane: Option<Pane>) -> Claude {
        Claude {
            record: Record {
                session_id: id.to_owned(),
                ..Record::default()
            },
            pane,
            latest: None,
        }
    }

    fn in_session(session: &str, window: u32) -> Pane {
        Pane {
            session: session.to_owned(),
            ..pane(window, window)
        }
    }

    #[test]
    fn follows_tmux_order_and_puts_outside_last() {
        let claudes = [
            claude("out", None),
            claude("b2", Some(in_session("b", 2))),
            claude("b1", Some(in_session("b", 1))),
        ];
        let groups = groups(&["a".to_owned(), "b".to_owned()], &claudes, &[]);
        let names: Vec<&GroupName> = groups.iter().map(|g| &g.name).collect();
        assert_eq!(
            names,
            [
                &GroupName::Session("a".into()),
                &GroupName::Session("b".into()),
                &GroupName::Outside
            ]
        );
        let b: Vec<&str> = groups
            .get(1)
            .into_iter()
            .flat_map(|g| &g.claudes)
            .map(|c| c.record.session_id.as_str())
            .collect();
        assert_eq!(b, ["b1", "b2"]);
    }

    #[test]
    fn lists_windows_without_claude_once_and_skips_sidebars() {
        let claudes = [claude("b1", Some(in_session("b", 1)))];
        let sidebar = Pane {
            sidebar: true,
            ..in_session("b", 2)
        };
        let second = Pane {
            pane_index: 7,
            ..in_session("b", 3)
        };
        let panes = [
            in_session("b", 1),
            sidebar,
            in_session("b", 3),
            second,
            in_session("a", 2),
        ];
        let groups = groups(&["a".to_owned(), "b".to_owned()], &claudes, &panes);
        let windows = |i: usize| -> Vec<u32> {
            groups
                .get(i)
                .into_iter()
                .flat_map(|g| &g.windows)
                .map(|p| p.window_index)
                .collect()
        };
        assert_eq!(windows(0), [2]);
        assert_eq!(windows(1), [3]);
    }
}

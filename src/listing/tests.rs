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

fn outside(id: &str, cwd: &str) -> Claude {
    Claude {
        record: Record {
            cwd: cwd.to_owned(),
            ..claude(id, None).record
        },
        ..claude(id, None)
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
fn sorts_outside_by_folder_then_id() {
    let claudes = [
        outside("3", "/x/web"),
        outside("2", "/y/api"),
        outside("1", "/x/web"),
    ];
    let groups = groups(&[], &claudes, &[]);
    let ids: Vec<&str> = groups
        .iter()
        .flat_map(|g| &g.claudes)
        .map(|c| c.record.session_id.as_str())
        .collect();
    assert_eq!(ids, ["2", "1", "3"]);
}

#[test]
fn sorts_a_path_without_a_folder_as_the_shown_question_mark() {
    let claudes = [outside("b", "/x/?"), outside("c", "/"), outside("a", "")];
    let groups = groups(&[], &claudes, &[]);
    let ids: Vec<&str> = groups
        .iter()
        .flat_map(|g| &g.claudes)
        .map(|c| c.record.session_id.as_str())
        .collect();
    assert_eq!(ids, ["a", "b", "c"]);
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

#[test]
fn a_linked_window_is_listed_once_under_its_last_session() {
    let panes = [in_session("a", 1), in_session("a", 2), in_session("b", 2)];
    let groups = groups(&["a".to_owned(), "b".to_owned()], &[], &panes);
    let windows = |i: usize| -> Vec<(String, u32)> {
        groups
            .get(i)
            .into_iter()
            .flat_map(|g| &g.windows)
            .map(|p| (p.session.clone(), p.window_index))
            .collect()
    };
    assert_eq!(windows(0), [("a".to_owned(), 1)]);
    assert_eq!(windows(1), [("b".to_owned(), 2)]);
}

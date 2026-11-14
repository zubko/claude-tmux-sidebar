use super::*;
use crate::session::Record;
use crate::tmux::pane::fixture::pane;

pub fn groups() -> Vec<Group> {
    let claude = Claude {
        record: Record {
            session_id: "c".to_owned(),
            prompt: "fix it".to_owned(),
            ..Record::default()
        },
        pane: None,
        latest: None,
    };
    vec![
        Group {
            name: GroupName::Session("a".to_owned()),
            claudes: vec![],
            windows: vec![],
        },
        Group {
            name: GroupName::Outside,
            claudes: vec![claude],
            windows: vec![],
        },
    ]
}

#[test]
fn lays_out_groups_with_gaps_and_placeholders() {
    let rows = rows(&groups());
    assert!(matches!(
        rows.as_slice(),
        [
            Row::Heading(GroupName::Session(_)),
            Row::Empty(_),
            Row::Gap,
            Row::Heading(GroupName::Outside),
            Row::Session(_),
            Row::Detail(_)
        ]
    ));
    assert_eq!(
        keys(&rows),
        [
            RowKey::Session("a".to_owned()),
            RowKey::Claude("c".to_owned())
        ]
    );
}

#[test]
fn maps_screen_lines_to_rows() {
    let rows = rows(&groups());
    let session = Some(RowKey::Session("a".to_owned()));
    assert_eq!(at(&rows, None, 20, FIRST_ROW), session);
    assert_eq!(at(&rows, None, 20, FIRST_ROW + 1), session);
    assert_eq!(at(&rows, None, 20, FIRST_ROW + 2), None);
    assert_eq!(at(&rows, None, 20, FIRST_ROW + 3), None);
    assert_eq!(
        at(&rows, None, 20, FIRST_ROW + 5),
        Some(RowKey::Claude("c".to_owned()))
    );
    assert_eq!(at(&rows, None, 4, FIRST_ROW + 4), None);
}

/// Claude `c1` in window 1 and `c3` in window 3; windows 2 and 4 run no Claude.
fn mixed_rows() -> Vec<Row> {
    let claude = |id: &str, window: u32| Claude {
        record: Record {
            session_id: id.to_owned(),
            ..Record::default()
        },
        pane: Some(pane(window, window)),
        latest: None,
    };
    rows(&[Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![claude("c1", 1), claude("c3", 3)],
        windows: vec![pane(2, 2), pane(4, 4)],
    }])
}

fn window(n: u32) -> RowKey {
    RowKey::Window(pane(n, n).window_id)
}

fn claude_key(id: &str) -> RowKey {
    RowKey::Claude(id.to_owned())
}

#[test]
fn puts_windows_without_claude_between_claude_sessions_in_window_order() {
    assert_eq!(
        keys(&mixed_rows()),
        [claude_key("c1"), window(2), claude_key("c3"), window(4)]
    );
}

#[test]
fn a_pane_is_stood_for_by_its_claude_then_its_window_then_its_session() {
    let rows = mixed_rows();
    let in_window_2 = Pane {
        pane_index: 5,
        ..pane(5, 2)
    };
    assert_eq!(key_for_pane(&rows, &in_window_2), window(2));
    assert_eq!(key_for_pane(&rows, &pane(6, 1)), claude_key("c1"));
    assert_eq!(
        key_for_pane(&rows, &pane(7, 7)),
        RowKey::Session("s".to_owned())
    );
}

mod scroll;

#[test]
fn the_cursor_passes_a_heading_with_rows_to_its_first_row() {
    let session = |name: &str| RowKey::Session(name.to_owned());
    assert_eq!(stop(&mixed_rows(), session("s")), claude_key("c1"));
    assert_eq!(stop(&rows(&groups()), session("a")), session("a"));
    assert_eq!(stop(&mixed_rows(), window(2)), window(2));
}

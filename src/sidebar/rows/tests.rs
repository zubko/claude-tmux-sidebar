use super::*;
use crate::session::Record;
use crate::tmux::pane::fixture::pane;

fn groups() -> Vec<Group> {
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
    assert_eq!(at(&rows, 20, FIRST_ROW), None);
    assert_eq!(
        at(&rows, 20, FIRST_ROW + 1),
        Some(RowKey::Session("a".to_owned()))
    );
    assert_eq!(at(&rows, 20, FIRST_ROW + 2), None);
    assert_eq!(
        at(&rows, 20, FIRST_ROW + 5),
        Some(RowKey::Claude("c".to_owned()))
    );
    assert_eq!(at(&rows, 4, FIRST_ROW + 4), None);
}

#[test]
fn puts_windows_without_claude_between_claude_sessions_in_window_order() {
    let claude = |id: &str, window: u32| Claude {
        record: Record {
            session_id: id.to_owned(),
            ..Record::default()
        },
        pane: Some(pane(window, window)),
        latest: None,
    };
    let group = Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![claude("c1", 1), claude("c3", 3)],
        windows: vec![pane(2, 2), pane(4, 4)],
    };
    let window = |n| RowKey::Window(pane(n, n).window_id);
    let claude_key = |id: &str| RowKey::Claude(id.to_owned());
    let rows = rows(&[group]);
    assert_eq!(
        keys(&rows),
        [claude_key("c1"), window(2), claude_key("c3"), window(4)]
    );
    let in_window_2 = Pane {
        pane_index: 5,
        ..pane(5, 2)
    };
    assert_eq!(key_for_pane(&rows, &in_window_2), window(2));
}

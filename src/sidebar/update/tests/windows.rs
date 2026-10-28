use super::*;
use crate::jump::Direction;
use crate::sidebar::selection::Selection;

/// Claude `a` in window 0, a window without Claude (pane 2) in window 1, Claude `b` in window 2.
fn groups() -> [Group; 1] {
    [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![
            claude("a", pane(1, 0), Status::Done),
            claude("b", pane(3, 2), Status::Done),
        ],
        windows: vec![pane(2, 1)],
    }]
}

#[test]
fn the_selection_follows_the_focus_into_a_window_without_claude() {
    let sidebar = Pane {
        sidebar: true,
        ..pane(9, 1)
    };
    let user = Pane {
        active: true,
        ..pane(2, 1)
    };
    let snapshot = crate::sidebar::snapshot::from_parts(
        rows::rows(&groups()),
        &[pane(1, 0), sidebar, user, pane(3, 2)],
        Some(pane(9, 1).id),
        false,
    );
    let model = loaded(Model::default(), snapshot);
    assert_eq!(
        selected(&model),
        Some(&RowKey::Window(pane(2, 1).window_id))
    );
}

#[test]
fn attention_jumps_skip_windows_without_claude() {
    let model = Model {
        snapshot: Snapshot {
            rows: rows::rows(&groups()),
            ..Snapshot::default()
        },
        selection: Selection {
            selected: Some(claude_key("a")),
            synced: None,
        },
        ..Model::default()
    };
    let next = update(model, Msg::Action(Action::Attention(Direction::Next))).0;
    assert_eq!(selected(&next), Some(&claude_key("b")));
    let back = update(next, Msg::Action(Action::Move(-1))).0;
    assert_eq!(selected(&back), Some(&RowKey::Window(pane(2, 1).window_id)));
}

use super::*;
use crate::sidebar::compact::Compact;
use crate::sidebar::selection::Selection;

/// With the cursor on Claude `a`, the rows are `s`, `a`, the detail line of `a` and `b`.
#[test]
fn clicks_in_a_compact_list_hit_the_row_drawn_there() {
    let model = loaded(
        Model {
            height: 20,
            compact: Compact::On,
            ..Model::default()
        },
        snapshot(false),
    );
    let model = update(model, Msg::Action(Action::First)).0;
    let (model, _) = update(
        model,
        Msg::Clicked {
            y: rows::FIRST_ROW + 3,
        },
    );
    assert_eq!(selected(&model), Some(&claude_key("b")));
}

/// With the cursor on Claude `b`, the rows are `s`, `a`, `b` and the detail line of `b`.
#[test]
fn the_selected_session_keeps_a_clickable_detail_line() {
    let model = loaded(
        Model {
            height: 20,
            compact: Compact::On,
            ..Model::default()
        },
        snapshot(false),
    );
    let model = update(model, Msg::Action(Action::Last)).0;
    let click = |line: u16| {
        let y = rows::FIRST_ROW + line;
        update(model.clone(), Msg::Clicked { y }).0
    };
    assert_eq!(selected(&click(1)), Some(&claude_key("a")));
    assert_eq!(selected(&click(2)), Some(&claude_key("b")));
    assert_eq!(selected(&click(3)), Some(&claude_key("b")));
}

/// Claude `a` in window 0 and a window without Claude in window 1; with the cursor on the window, the rows
/// are `s`, `a`, the window and its blank line.
#[test]
fn a_click_on_the_blank_line_selects_its_window() {
    let window = RowKey::Window(pane(2, 1).window_id);
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![claude("a", pane(1, 0), Status::Done)],
        windows: vec![pane(2, 1)],
    }];
    let model = Model {
        height: 20,
        compact: Compact::On,
        snapshot: Snapshot {
            rows: rows::rows(&groups),
            ..Snapshot::default()
        },
        selection: Selection {
            selected: Some(window.clone()),
            synced: None,
        },
        ..Model::default()
    };
    let y = rows::FIRST_ROW + 3;
    assert_eq!(
        selected(&update(model, Msg::Clicked { y }).0),
        Some(&window)
    );
}

use super::*;
use crate::sidebar::compact::Compact;

pub(super) fn window(index: u32, name: &str) -> Pane {
    Pane {
        window_name: name.to_owned(),
        session_windows: 3,
        ..pane(index, index)
    }
}

/// Claude `a` in window 2; windows 1 and 3 run no Claude; the sidebar is in window 3.
pub(super) fn model_with_windows(selected: Option<RowKey>) -> Model {
    let groups = [Group {
        name: GroupName::Session("work".to_owned()),
        claudes: vec![claude_in(3)],
        windows: vec![window(1, "talk to core"), window(3, "docs")],
    }];
    Model {
        snapshot: Snapshot {
            rows: rows::rows(&groups),
            window: Some(pane(3, 3).window_id),
            ..Snapshot::default()
        },
        selection: Selection {
            selected,
            synced: None,
        },
        now: 102,
        height: 8,
        help: true,
        ..Model::default()
    }
}

#[test]
fn windows_without_claude_sit_between_sessions_in_window_order() {
    let screen = screen(&model_with_windows(None), &EN);
    let list: Vec<&str> = screen
        .iter()
        .skip(2)
        .take(5)
        .map(|s| s.trim_end())
        .collect();
    assert_eq!(
        list,
        [
            "▾ work",
            "  1:talk to core",
            "✓ 2:api                        2s",
            "└",
            "  3:docs",
        ]
    );
}

#[test]
fn the_window_you_are_in_has_the_active_background() {
    let model = model_with_windows(None);
    let bgs = backgrounds(&model, 6);
    assert!(!bgs.is_empty() && bgs.iter().all(|bg| *bg == Some(ACTIVE)));
    assert!(backgrounds(&model, 3).iter().all(Option::is_none));
}

#[test]
fn a_selected_window_row_gets_the_cursor() {
    let key = RowKey::Window(window(1, "").window_id);
    let model = model_with_windows(Some(key));
    let line = lines(&model, &EN, WIDTH, model.height)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert_eq!(line.width(), usize::from(WIDTH));
    assert!(line.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
}

#[test]
fn the_cursor_on_the_window_you_are_in_shows_its_number_in_accent() {
    let key = RowKey::Window(window(3, "").window_id);
    let model = model_with_windows(Some(key));
    let line = lines(&model, &EN, WIDTH, model.height)
        .get(6)
        .cloned()
        .unwrap_or_default();
    assert!(line.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    let accent: Vec<&str> = line
        .spans
        .iter()
        .filter(|s| s.style.fg == Some(ACCENT))
        .map(|s| s.content.as_ref())
        .collect();
    assert_eq!(accent, ["3:"]);
}

#[test]
fn a_window_row_without_a_name_shows_its_index() {
    let model = model_with_windows(None);
    let pane = Pane {
        session_windows: 1,
        ..window(1, "")
    };
    let line = text(&window::line(&model, &pane, usize::from(WIDTH), false));
    assert_eq!(line, "  1:");
}

#[test]
fn a_selected_window_in_a_compact_list_has_a_blank_line_under_the_cursor() {
    let key = RowKey::Window(window(1, "").window_id);
    let model = Model {
        compact: Compact::On,
        ..model_with_windows(Some(key))
    };
    let line = lines(&model, &EN, WIDTH, model.height)
        .get(4)
        .cloned()
        .unwrap_or_default();
    assert_eq!(text(&line).trim(), "");
    assert_eq!(line.width(), usize::from(WIDTH));
    assert!(line.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
}

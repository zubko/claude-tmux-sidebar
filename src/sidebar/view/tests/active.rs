use super::*;

fn here(mut model: Model) -> Model {
    model.snapshot.window = Some(pane(1, 2).window_id);
    model
}

fn is_active(model: &Model, row: usize) -> bool {
    let bgs = backgrounds(model, row);
    !bgs.is_empty() && bgs.iter().all(|bg| *bg == Some(ACTIVE))
}

fn is_cursor(model: &Model, row: usize) -> bool {
    let bgs = backgrounds(model, row);
    !bgs.is_empty() && bgs.iter().all(|bg| *bg == Some(CURSOR))
}

fn is_plain(model: &Model, row: usize) -> bool {
    backgrounds(model, row).iter().all(Option::is_none)
}

fn accent_spans(model: &Model, row: usize) -> Vec<String> {
    let lines = lines(model, &EN, WIDTH, model.height);
    let line = lines.get(row).cloned().unwrap_or_default();
    line.spans
        .iter()
        .filter(|s| s.style.fg == Some(ACCENT))
        .map(|s| s.content.to_string())
        .collect()
}

/// The user's pane 5 is in window 7, which has no row; `work` is its tmux session.
fn heading_stands_in(selected: bool) -> Model {
    let mut model = model(Status::Done, "", false);
    let focus = Pane {
        session: "work".to_owned(),
        ..pane(5, 7)
    };
    model.snapshot.window = Some(focus.window_id);
    model.snapshot.focus = Some(focus);
    model.selection.selected = selected.then(|| RowKey::Session("work".to_owned()));
    model
}

#[test]
fn the_session_row_and_its_detail_row_have_the_active_background() {
    let model = here(model(Status::Done, "", false));
    assert!(is_active(&model, 3) && is_active(&model, 4));
    assert_eq!(
        screen(&model, &EN).get(3).map(String::as_str),
        Some("✓ api                          2s ")
    );
    assert!(is_plain(&model, 0) && is_plain(&model, 2));
}

#[test]
fn rows_of_other_windows_have_no_background() {
    let model = model(Status::Done, "", false);
    assert!(is_plain(&model, 2) && is_plain(&model, 3) && is_plain(&model, 4));
}

#[test]
fn the_cursor_wins_over_active_and_the_window_number_takes_the_accent() {
    let model = here(model(Status::Done, "", true));
    assert!(is_cursor(&model, 3) && is_cursor(&model, 4));
    assert_eq!(accent_spans(&model, 3), ["api"]);
    let several = here(model_of(claude_in(3), true));
    assert_eq!(accent_spans(&several, 3), ["2:"]);
}

#[test]
fn the_accent_is_off_without_the_cursor_or_outside_the_window() {
    let model = here(model_of(claude_in(3), false));
    assert!(accent_spans(&model, 3).is_empty());
    let elsewhere = model_of(claude_in(3), true);
    assert!(accent_spans(&elsewhere, 3).is_empty());
}

#[test]
fn the_heading_stands_for_this_window_when_it_has_no_row() {
    let model = heading_stands_in(false);
    assert!(is_active(&model, 2));
    assert!(is_plain(&model, 3));
    assert_eq!(
        screen(&model, &EN).get(2).map(|s| s.trim_end()),
        Some("▾ work")
    );
    assert!(accent_spans(&model, 2).is_empty());
}

#[test]
fn the_cursor_on_the_stand_in_heading_turns_it_accent() {
    let model = heading_stands_in(true);
    assert!(is_cursor(&model, 2));
    assert_eq!(accent_spans(&model, 2), ["▾ work"]);
}

use super::windows::{model_with_windows, window};
use super::*;
use crate::sidebar::mark::ActiveMark;

fn bar(model: Model) -> Model {
    Model {
        mark: ActiveMark::Bar,
        ..model
    }
}

fn here(mut model: Model) -> Model {
    model.snapshot.window = Some(pane(1, 2).window_id);
    model
}

/// The color of the left column on screen row `row`.
fn first_span_color(model: &Model, row: usize) -> Option<Color> {
    let lines = lines(model, &EN, WIDTH, model.height);
    lines
        .get(row)
        .and_then(|l| l.spans.first())
        .and_then(|s| s.style.fg)
}

#[test]
fn the_bar_takes_the_left_column_on_every_line() {
    let screen = screen(&bar(model(Status::Waiting, "Edit", false)), &EN);
    assert_eq!(screen.first().map(String::as_str), Some(" Claude ◆1 "));
    assert_eq!(screen.get(2).map(String::as_str), Some(" ▾ work"));
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ◆ api                    Edit 2s ")
    );
    assert_eq!(
        screen.last().map(String::as_str),
        Some(" ⏎ jump  j/k move  q close")
    );
}

#[test]
fn the_rows_of_this_window_get_the_bar_and_no_background() {
    let model = here(bar(model(Status::Done, "", false)));
    let screen = screen(&model, &EN);
    assert!(screen.get(3).is_some_and(|s| s.starts_with("▌✓ api ")));
    assert!(screen.get(4).is_some_and(|s| s.starts_with("▌└")));
    assert!(backgrounds(&model, 3).iter().all(Option::is_none));
    assert!(backgrounds(&model, 4).iter().all(Option::is_none));
    assert_eq!(first_span_color(&model, 3), Some(ACCENT));
    assert_eq!(first_span_color(&model, 4), Some(ACCENT));
}

#[test]
fn rows_of_other_windows_get_a_blank_column() {
    let model = bar(model(Status::Done, "", false));
    let screen = screen(&model, &EN);
    assert!(screen.get(3).is_some_and(|s| s.starts_with(" ✓ api ")));
    assert_eq!(first_span_color(&model, 3), None);
}

#[test]
fn the_cursor_keeps_the_index_plain_with_the_bar() {
    let model = here(bar(model_of(claude_in(3), true)));
    let lines = lines(&model, &EN, WIDTH, model.height);
    let row = lines.get(3).cloned().unwrap_or_default();
    assert_eq!(row.width(), usize::from(WIDTH));
    assert!(row.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    let accented: Vec<&str> = row
        .spans
        .iter()
        .filter(|s| s.style.fg == Some(ACCENT))
        .map(|s| s.content.as_ref())
        .collect();
    assert_eq!(accented, ["▌"]);
}

#[test]
fn the_stand_in_heading_gets_a_bar_in_the_text_color() {
    let model = bar(heading_stands_in(false));
    assert_eq!(
        screen(&model, &EN).get(2).map(String::as_str),
        Some("▌▾ work")
    );
    assert_eq!(first_span_color(&model, 2), None);
    assert!(backgrounds(&model, 2).iter().all(Option::is_none));
    let selected = bar(heading_stands_in(true));
    let row = lines(&selected, &EN, WIDTH, selected.height)
        .get(2)
        .cloned()
        .unwrap_or_default();
    assert!(row.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    assert!(row.spans.iter().all(|s| s.style.fg != Some(ACCENT)));
    assert_eq!(text(&row).trim_end(), "▌▾ work");
}

#[test]
fn the_placeholder_shares_the_bar_of_its_heading() {
    let groups = [Group {
        name: GroupName::Session("work".to_owned()),
        claudes: vec![],
        windows: vec![],
    }];
    let focus = Pane {
        session: "work".to_owned(),
        ..pane(5, 7)
    };
    let model = bar(Model {
        snapshot: Snapshot {
            rows: rows::rows(&groups),
            window: Some(focus.window_id),
            focus: Some(focus),
            ..Snapshot::default()
        },
        height: 8,
        help: true,
        ..Model::default()
    });
    let screen = screen(&model, &EN);
    assert_eq!(screen.get(2).map(String::as_str), Some("▌▾ work"));
    assert_eq!(screen.get(3).map(String::as_str), Some("▌  none"));
    assert_eq!(first_span_color(&model, 3), None);
}

#[test]
fn a_window_row_of_this_window_gets_the_bar() {
    let model = bar(model_with_windows(None));
    let screen = screen(&model, &EN);
    assert!(
        screen
            .get(3)
            .is_some_and(|s| s.starts_with("   1:talk to core"))
    );
    assert_eq!(screen.get(6).map(String::as_str), Some("▌  3:docs"));
    assert_eq!(first_span_color(&model, 6), Some(ACCENT));
    let key = RowKey::Window(window(3, "").window_id);
    let selected = bar(model_with_windows(Some(key)));
    let row = lines(&selected, &EN, WIDTH, selected.height)
        .get(6)
        .cloned()
        .unwrap_or_default();
    assert!(row.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    assert!(row.spans.iter().skip(1).all(|s| s.style.fg != Some(ACCENT)));
}

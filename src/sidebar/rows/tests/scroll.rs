use super::*;

/// A heading and ten windows without Claude: eleven rows.
fn long_list() -> Vec<Row> {
    let windows = (0..10)
        .map(|i| Pane {
            session: "s".to_owned(),
            ..pane(i, i)
        })
        .collect();
    rows(&[Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
        windows,
    }])
}

fn last_key(rows: &[Row]) -> Option<RowKey> {
    keys(rows).last().cloned()
}

#[test]
fn a_list_that_fits_does_not_scroll() {
    let rows = long_list();
    let last = last_key(&rows);
    assert_eq!(offset(&rows, last.as_ref(), FIRST_ROW + 11), 0);
    assert_eq!(offset(&rows, None, FIRST_ROW + 3), 0);
}

#[test]
fn the_list_scrolls_just_enough_to_show_the_selected_row() {
    let rows = long_list();
    let last = last_key(&rows);
    let end = FIRST_ROW + 5;
    assert_eq!(offset(&rows, last.as_ref(), end), 6);
    let first = keys(&rows).first().cloned();
    assert_eq!(offset(&rows, first.as_ref(), end), 0);
}

#[test]
fn a_click_lands_on_the_row_drawn_after_scrolling() {
    let rows = long_list();
    let last = last_key(&rows);
    let end = FIRST_ROW + 5;
    assert_eq!(at(&rows, last.as_ref(), end, end - 1), last);
}

#[test]
fn the_detail_line_scrolls_in_with_its_session() {
    let rows = rows(&groups());
    let claude = RowKey::Claude("c".to_owned());
    assert_eq!(offset(&rows, Some(&claude), FIRST_ROW + 2), 4);
    assert_eq!(offset(&rows, Some(&claude), FIRST_ROW + 3), 3);
}

#[test]
fn a_single_line_list_keeps_the_session_over_its_detail() {
    let rows = rows(&groups());
    let claude = RowKey::Claude("c".to_owned());
    assert_eq!(offset(&rows, Some(&claude), FIRST_ROW + 1), 4);
}

#[test]
fn a_heading_scrolls_in_with_its_placeholder() {
    let rows = rows(&groups());
    let session = RowKey::Session("a".to_owned());
    assert_eq!(offset(&rows, Some(&session), FIRST_ROW + 1), 0);
    assert_eq!(
        at(&rows, Some(&session), FIRST_ROW + 1, FIRST_ROW),
        Some(session)
    );
}

#[test]
fn a_window_scrolls_in_with_its_blank_line() {
    let window = Row::Window(pane(1, 1));
    let key = key(&window);
    let rows = [
        Row::Heading(GroupName::Session("s".to_owned())),
        window.clone(),
        Row::Pad(Box::new(window)),
    ];
    assert_eq!(offset(&rows, key.as_ref(), FIRST_ROW + 2), 1);
    assert_eq!(at(&rows, key.as_ref(), FIRST_ROW + 2, FIRST_ROW + 1), key);
}

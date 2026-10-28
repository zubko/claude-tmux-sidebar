use super::*;

fn window(index: u32, name: &str) -> Pane {
    Pane {
        window_name: name.to_owned(),
        session_windows: 3,
        ..pane(index, index)
    }
}

fn model_with_windows(selected: Option<RowKey>) -> Model {
    let claude = Claude {
        record: record(Status::Done, ""),
        pane: Some(Pane {
            window_name: "api".to_owned(),
            session_windows: 3,
            ..pane(1, 2)
        }),
        latest: None,
    };
    let groups = [Group {
        name: GroupName::Session("work".to_owned()),
        claudes: vec![claude],
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
    let list: Vec<&str> = screen.iter().skip(2).take(4).map(String::as_str).collect();
    assert_eq!(
        list,
        [
            " ▾ work",
            "   1:talk to core",
            " ✓ 2:api                       2s ",
            "▌  3:docs",
        ]
    );
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
fn a_window_row_without_a_name_shows_its_index() {
    let model = model_with_windows(None);
    let pane = Pane {
        session_windows: 1,
        ..window(1, "")
    };
    let line = text(&crate::sidebar::view::session::window(
        &model,
        &pane,
        usize::from(WIDTH),
    ));
    assert_eq!(line, "   1:");
}

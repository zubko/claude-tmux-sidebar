use super::*;

/// Session `other` lists nothing; in `s`, Claudes `a` and `b` run in windows 0 and 1.
fn groups() -> [Group; 2] {
    let session = |name: &str, claudes| Group {
        name: GroupName::Session(name.to_owned()),
        claudes,
        windows: vec![],
    };
    [
        session("other", vec![]),
        session(
            "s",
            vec![
                claude("a", pane(1, 0), Status::Done),
                claude("b", pane(2, 1), Status::Done),
            ],
        ),
    ]
}

#[test]
fn clicking_a_heading_switches_to_its_session_and_puts_the_cursor_on_its_first_row() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(false),
    );
    let (model, effects) = update(model, Msg::Clicked { y: rows::FIRST_ROW });
    assert_eq!(selected(&model), Some(&claude_key("a")));
    assert!(matches!(
        effects.as_slice(),
        [Effect::Goto {
            target: Target::Session(name),
            ..
        }] if name == "s"
    ));
}

/// Window 3 of `s` holds the sidebar (pane 9) and the user's pane 4.
#[test]
fn the_selection_follows_the_focus_to_the_first_row_under_its_heading() {
    let sidebar = Pane {
        sidebar: true,
        ..pane(9, 3)
    };
    let user = Pane {
        active: true,
        ..pane(4, 3)
    };
    let snapshot = crate::sidebar::snapshot::from_parts(
        rows::rows(&groups()),
        &[pane(1, 0), pane(2, 1), sidebar, user],
        Some(pane(9, 3).id),
        false,
    );
    let model = loaded(Model::default(), snapshot);
    assert_eq!(selected(&model), Some(&claude_key("a")));
}

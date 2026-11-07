use super::*;

fn session_key() -> RowKey {
    RowKey::Session("s".to_owned())
}

#[test]
fn clicking_a_heading_switches_to_its_session() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(false),
    );
    let (model, effects) = update(model, Msg::Clicked { y: rows::FIRST_ROW });
    assert_eq!(selected(&model), Some(&session_key()));
    assert!(matches!(
        effects.as_slice(),
        [Effect::Goto {
            target: Target::Session(name),
            ..
        }] if name == "s"
    ));
}

/// Claudes `a` and `b` run in windows 0 and 1; window 3 holds the sidebar (pane 9) and the user's pane 4.
#[test]
fn the_selection_follows_the_focus_to_the_heading_when_the_window_has_no_row() {
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![
            claude("a", pane(1, 0), Status::Done),
            claude("b", pane(2, 1), Status::Done),
        ],
        windows: vec![],
    }];
    let sidebar = Pane {
        sidebar: true,
        ..pane(9, 3)
    };
    let user = Pane {
        active: true,
        ..pane(4, 3)
    };
    let snapshot = crate::sidebar::snapshot::from_parts(
        rows::rows(&groups),
        &[pane(1, 0), pane(2, 1), sidebar, user],
        Some(pane(9, 3).id),
        false,
    );
    let model = loaded(Model::default(), snapshot);
    assert_eq!(selected(&model), Some(&session_key()));
}

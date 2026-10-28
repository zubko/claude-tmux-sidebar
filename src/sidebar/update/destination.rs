//! Where activating a row goes.

use super::Target;
use crate::sidebar::rows::{Row, RowKey};
use crate::sidebar::snapshot::Snapshot;
use crate::tmux::pane::WindowId;

/// The target of the row `key` and the window it is in.
pub fn destination(snapshot: &Snapshot, key: &RowKey) -> Option<(Target, Option<WindowId>)> {
    match key {
        RowKey::Session(name) => Some((
            Target::Session(name.clone()),
            snapshot.session_windows.get(name).copied(),
        )),
        RowKey::Window(window) => snapshot.rows.iter().find_map(|r| match r {
            Row::Window(p) if p.window_id == *window => Some((Target::Pane(p.id), Some(*window))),
            _ => None,
        }),
        RowKey::Claude(id) => snapshot.rows.iter().find_map(|r| match r {
            Row::Session(c) if c.record.session_id == *id => c
                .pane
                .as_ref()
                .map(|p| (Target::Pane(p.id), Some(p.window_id))),
            _ => None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Action, Effect, Model, Msg, update};
    use super::*;
    use crate::listing::{Group, GroupName};
    use crate::sidebar::rows::{self, RowKey};
    use crate::tmux::pane::fixture::pane;

    #[test]
    fn activating_a_window_row_goes_to_its_pane() {
        let groups = [Group {
            name: GroupName::Session("s".to_owned()),
            claudes: vec![],
            windows: vec![pane(4, 4)],
        }];
        let window = pane(4, 4).window_id;
        let model = Model {
            snapshot: Snapshot {
                rows: rows::rows(&groups),
                window: Some(pane(1, 1).window_id),
                ..Snapshot::default()
            },
            ..Model::default()
        };
        let model = update(model, Msg::Action(Action::First)).0;
        assert_eq!(model.selection.selected, Some(RowKey::Window(window)));
        let (_, effects) = update(model, Msg::Action(Action::Activate));
        assert_eq!(
            effects,
            [Effect::Goto {
                target: Target::Pane(pane(4, 4).id),
                leaves_window: true
            }]
        );
    }
}

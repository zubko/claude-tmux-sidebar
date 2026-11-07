//! What the sidebar knows about tmux and Claude at one moment.

use std::collections::HashMap;
use std::path::Path;

use super::rows::{self, Row};
use crate::context::Context;
use crate::listing;
use crate::session::context_window::ContextWindows;
use crate::session::live::{self, Claude};
use crate::session::seed::Seeded;
use crate::session::transcript::cache::Transcripts;
use crate::tmux::{
    self,
    pane::{self, Pane, PaneId, WindowId},
};

const CONTEXT_WINDOWS_OPTION: &str = "@cts_context_windows";
const ALL_WINDOWS_OPTION: &str = "@cts_all_windows";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub rows: Vec<Row>,
    /// The window the sidebar belongs to (or the popup was opened from).
    pub window: Option<WindowId>,
    /// The pane the user works in, in that window.
    pub focus: Option<Pane>,
    /// The sidebar pane itself has the focus.
    pub sidebar_active: bool,
    /// This sidebar should go: its window has no real pane left, or an older sidebar (concurrent hooks can add two).
    pub redundant: bool,
    /// Each tmux session's current window.
    pub session_windows: HashMap<String, WindowId>,
    /// Context window sizes by model (`@cts_context_windows`).
    pub context_windows: ContextWindows,
}

/// What the loop keeps from one load to the next.
#[derive(Debug, Default)]
pub struct Caches {
    pub transcripts: Transcripts,
    pub seeded: Seeded,
}

/// Reads tmux, the state files and the transcripts that changed since the last load.
/// `me` is the sidebar pane, or the pane a popup was opened from.
pub fn load(ctx: &Context, me: Option<PaneId>, popup: bool, caches: Caches) -> (Snapshot, Caches) {
    let panes = tmux::panes();
    let seeded = caches.seeded.seed(ctx, &panes);
    let claudes = live::load(&ctx.state_dir, &ctx.socket, &panes);
    let (claudes, transcripts) = with_transcripts(claudes, caches.transcripts);
    let shown = window_panes(&panes, tmux::global_option(ALL_WINDOWS_OPTION).as_deref());
    let groups = listing::groups(&tmux::sessions(), &claudes, shown);
    let context_windows = tmux::global_option(CONTEXT_WINDOWS_OPTION)
        .map(|list| ContextWindows::parse(&list))
        .unwrap_or_default();
    let snapshot = Snapshot {
        context_windows,
        ..from_parts(rows::rows(&groups), &panes, me, popup)
    };
    (
        snapshot,
        Caches {
            transcripts,
            seeded,
        },
    )
}

/// Re-reads the transcripts that changed and gives each session what its transcript says.
fn with_transcripts(claudes: Vec<Claude>, transcripts: Transcripts) -> (Vec<Claude>, Transcripts) {
    let transcripts = transcripts.refresh(
        claudes
            .iter()
            .map(|c| Path::new(c.record.transcript.as_str()))
            .filter(|p| !p.as_os_str().is_empty()),
    );
    let claudes = claudes
        .into_iter()
        .map(|c| Claude {
            latest: transcripts.latest(&c.record.transcript),
            ..c
        })
        .collect();
    (claudes, transcripts)
}

/// The panes whose windows are listed even without Claude: all of them with `@cts_all_windows on`,
/// else none.
fn window_panes<'a>(panes: &'a [Pane], option: Option<&str>) -> &'a [Pane] {
    if option == Some("on") { panes } else { &[] }
}

/// The pure part of [`load`].
pub fn from_parts(rows: Vec<Row>, panes: &[Pane], me: Option<PaneId>, popup: bool) -> Snapshot {
    let mine = me.and_then(|id| panes.iter().find(|p| p.id == id));
    let window = mine.map(|p| p.window_id);
    Snapshot {
        rows,
        window,
        focus: window.and_then(|w| pane::focused(panes, w)).cloned(),
        sidebar_active: !popup && mine.is_some_and(|p| p.active),
        redundant: !popup && mine.is_some_and(|p| redundant(panes, p)),
        session_windows: panes
            .iter()
            .filter(|p| p.window_active)
            .map(|p| (p.session.clone(), p.window_id))
            .collect(),
        context_windows: ContextWindows::default(),
    }
}

fn redundant(panes: &[Pane], me: &Pane) -> bool {
    let window: Vec<&Pane> = panes
        .iter()
        .filter(|p| p.window_id == me.window_id)
        .collect();
    let oldest_sidebar = window.iter().filter(|p| p.sidebar).map(|p| p.id).min();
    window.iter().all(|p| p.sidebar) || oldest_sidebar != Some(me.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::pane::fixture::pane;

    fn sidebar(id: u32) -> Pane {
        Pane {
            sidebar: true,
            ..pane(id, 0)
        }
    }

    #[test]
    fn a_lone_or_newer_sidebar_is_redundant() {
        let me = sidebar(5);
        assert!(from_parts(vec![], std::slice::from_ref(&me), Some(me.id), false).redundant);
        assert!(
            from_parts(
                vec![],
                &[sidebar(4), me.clone(), pane(1, 0)],
                Some(me.id),
                false
            )
            .redundant
        );
        assert!(!from_parts(vec![], &[me.clone(), pane(1, 0)], Some(me.id), false).redundant);
    }

    #[test]
    fn lists_windows_without_claude_only_when_the_option_is_on() {
        let panes = [pane(1, 0)];
        assert_eq!(window_panes(&panes, Some("on")), &panes);
        assert!(window_panes(&panes, Some("off")).is_empty());
        assert!(window_panes(&panes, None).is_empty());
    }

    #[test]
    fn popups_never_count_as_active_or_redundant() {
        let origin = Pane {
            active: true,
            ..pane(1, 0)
        };
        let snapshot = from_parts(vec![], std::slice::from_ref(&origin), Some(origin.id), true);
        assert!(!snapshot.sidebar_active && !snapshot.redundant);
        assert_eq!(snapshot.focus.map(|p| p.id), Some(origin.id));
    }
}

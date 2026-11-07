//! tmux panes as parsed from `list-panes`, and the ids that name them.

use std::fmt;

/// A pane id such as `%12`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaneId(u32);

/// A window id such as `@3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(u32);

impl PaneId {
    pub fn parse(s: &str) -> Option<Self> {
        s.strip_prefix('%')?.parse().ok().map(Self)
    }
}

impl WindowId {
    pub fn parse(s: &str) -> Option<Self> {
        s.strip_prefix('@')?.parse().ok().map(Self)
    }
}

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

/// One pane of the tmux server.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // mirrors tmux's own flags
pub struct Pane {
    pub id: PaneId,
    /// The process the pane runs, usually its shell.
    pub pid: u32,
    pub session: String,
    pub window_index: u32,
    pub pane_index: u32,
    /// How many windows the pane's session has.
    pub session_windows: u32,
    pub window_id: WindowId,
    pub window_name: String,
    /// The name was set by hand: tmux turns `automatic-rename` off for such a window.
    pub window_named: bool,
    pub sidebar: bool,
    pub active: bool,
    pub last: bool,
    pub window_active: bool,
}

/// The `list-panes -F` format that [`parse`] reads.
pub const FORMAT: &str = "#{pane_id}\t#{pane_pid}\t#{session_name}\t#{window_index}\t#{pane_index}\t#{window_id}\t\
                          #{@cts_sidebar}\t#{pane_active}\t#{pane_last}\t#{window_active}\t#{session_windows}\t#{automatic-rename}\t#{window_name}";

/// Parses `list-panes -F FORMAT` output, skipping malformed lines.
pub fn parse(output: &str) -> Vec<Pane> {
    output.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<Pane> {
    // Last field and splitn: a window name may contain a tab.
    let fields: Vec<&str> = line.splitn(13, '\t').collect();
    let [
        id,
        pid,
        session,
        window_index,
        pane_index,
        window_id,
        bools @ ..,
        session_windows,
        automatic_rename,
        window_name,
    ] = fields.as_slice()
    else {
        return None;
    };
    Some(Pane {
        id: PaneId::parse(id)?,
        pid: pid.parse().ok()?,
        session: (*session).to_owned(),
        window_index: window_index.parse().ok()?,
        pane_index: pane_index.parse().ok()?,
        session_windows: session_windows.parse().ok()?,
        window_id: WindowId::parse(window_id)?,
        window_name: (*window_name).to_owned(),
        window_named: *automatic_rename == "0",
        ..flags(bools)?
    })
}

/// The `#{@cts_sidebar}`, `#{pane_active}`, `#{pane_last}` and `#{window_active}` fields.
fn flags(fields: &[&str]) -> Option<Pane> {
    let [sidebar, active, last, window_active] = fields else {
        return None;
    };
    Some(Pane {
        sidebar: *sidebar == "1",
        active: *active == "1",
        last: *last == "1",
        window_active: *window_active == "1",
        ..Pane::default()
    })
}

/// The pane the user works in: the active non-sidebar pane, or the previous one while the sidebar has the focus.
pub fn focused(panes: &[Pane], window: WindowId) -> Option<&Pane> {
    let real: Vec<&Pane> = panes
        .iter()
        .filter(|p| p.window_id == window && !p.sidebar)
        .collect();
    real.iter()
        .find(|p| p.active)
        .or_else(|| real.iter().find(|p| p.last))
        .or_else(|| real.first())
        .copied()
}

/// The sidebar pane of a window, if it has one.
pub fn sidebar_of(panes: &[Pane], window: WindowId) -> Option<&Pane> {
    panes.iter().find(|p| p.window_id == window && p.sidebar)
}

#[cfg(test)]
pub mod fixture {
    use super::{Pane, PaneId, WindowId};

    /// A pane of session `s` with pane index equal to its id.
    pub fn pane(id: u32, window: u32) -> Pane {
        Pane {
            id: PaneId(id),
            pid: 0,
            session: "s".to_owned(),
            window_index: window,
            pane_index: id,
            session_windows: 1,
            window_id: WindowId(window),
            window_name: String::new(),
            window_named: true,
            sidebar: false,
            active: false,
            last: false,
            window_active: true,
        }
    }
}

#[cfg(test)]
mod tests;

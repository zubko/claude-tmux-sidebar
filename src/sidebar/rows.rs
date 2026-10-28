//! The list the sidebar shows: one row per group heading, Claude session, its detail line, window
//! without Claude, placeholder or gap.

use itertools::Itertools;

use crate::listing::{Group, GroupName};
use crate::session::live::Claude;
use crate::tmux::pane::{Pane, WindowId};

/// The first screen row of the list (below the title and the rule).
pub const FIRST_ROW: u16 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    Gap,
    Heading(GroupName),
    Session(Claude),
    /// The line under a Claude session: its last prompt and context usage.
    Detail(Claude),
    /// A window without Claude, by one of its panes.
    Window(Pane),
    /// The placeholder under a tmux session without Claude; it stands for that session.
    Empty(String),
}

/// A selectable row: a tmux session without windows (its placeholder), a window without Claude, or a
/// Claude session by id.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RowKey {
    Session(String),
    Window(WindowId),
    Claude(String),
}

pub fn rows(groups: &[Group]) -> Vec<Row> {
    groups
        .iter()
        .enumerate()
        .flat_map(|(i, g)| {
            let gap = (i > 0).then_some(Row::Gap);
            let empty = match &g.name {
                GroupName::Session(name) if g.claudes.is_empty() && g.windows.is_empty() => {
                    Some(Row::Empty(name.clone()))
                }
                _ => None,
            };
            gap.into_iter()
                .chain([Row::Heading(g.name.clone())])
                .chain(entries(g))
                .chain(empty)
        })
        .collect()
}

/// Claude sessions and windows without Claude, in window order.
fn entries(group: &Group) -> impl Iterator<Item = Row> {
    let claudes = group.claudes.iter().map(|c| {
        let window = c.pane.as_ref().map_or(0, |p| p.window_index);
        (window, session_rows(c))
    });
    let windows = group
        .windows
        .iter()
        .map(|p| (p.window_index, vec![Row::Window(p.clone())]));
    claudes
        .chain(windows)
        .sorted_by_key(|(window, _)| *window)
        .flat_map(|(_, rows)| rows)
}

/// A Claude session, followed by its detail line when there is anything to show.
fn session_rows(claude: &Claude) -> Vec<Row> {
    let detail = !claude.record.prompt.is_empty() || claude.context().is_some();
    [Row::Session(claude.clone())]
        .into_iter()
        .chain(detail.then(|| Row::Detail(claude.clone())))
        .collect()
}

/// The key of a selectable row: the cursor stops on Claude sessions, windows without Claude, and the
/// placeholder of a tmux session without windows, never on a heading.
pub fn key(row: &Row) -> Option<RowKey> {
    match row {
        Row::Empty(name) => Some(RowKey::Session(name.clone())),
        Row::Window(pane) => Some(RowKey::Window(pane.window_id)),
        Row::Session(c) => Some(RowKey::Claude(c.record.session_id.clone())),
        Row::Heading(_) | Row::Detail(_) | Row::Gap => None,
    }
}

/// The selectable row a line belongs to: a detail line belongs to its session.
pub fn owner(row: &Row) -> Option<RowKey> {
    match row {
        Row::Detail(c) => Some(RowKey::Claude(c.record.session_id.clone())),
        _ => key(row),
    }
}

/// Selectable rows in display order.
pub fn keys(rows: &[Row]) -> Vec<RowKey> {
    rows.iter().filter_map(key).collect()
}

/// The row that stands for a pane: its Claude session, its window, or else its tmux session.
pub fn key_for_pane(rows: &[Row], pane: &Pane) -> RowKey {
    rows.iter()
        .find_map(|r| match r {
            Row::Session(c) if c.record.pane_id() == Some(pane.id) => {
                Some(RowKey::Claude(c.record.session_id.clone()))
            }
            Row::Window(p) if p.window_id == pane.window_id => Some(RowKey::Window(p.window_id)),
            _ => None,
        })
        .unwrap_or_else(|| RowKey::Session(pane.session.clone()))
}

/// Rows with the screen line they are drawn on, above screen line `end`.
pub fn placed(rows: &[Row], end: u16) -> impl Iterator<Item = (u16, &Row)> {
    (FIRST_ROW..end).zip(rows)
}

/// The selectable row drawn on screen line `y`.
pub fn at(rows: &[Row], end: u16, y: u16) -> Option<RowKey> {
    placed(rows, end)
        .find(|&(line, _)| line == y)
        .and_then(|(_, row)| owner(row))
}

#[cfg(test)]
mod tests;

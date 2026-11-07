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
    /// The placeholder under a tmux session with nothing listed; it belongs to the heading above it.
    Empty(String),
}

/// A selectable row: a tmux session (its heading), a window without Claude, or a Claude session by id.
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

/// A Claude session and its detail line, so that every session takes two lines.
fn session_rows(claude: &Claude) -> Vec<Row> {
    vec![Row::Session(claude.clone()), Row::Detail(claude.clone())]
}

/// The key of a selectable row: the cursor stops on tmux session headings, Claude sessions and windows
/// without Claude, never on the heading outside tmux.
pub fn key(row: &Row) -> Option<RowKey> {
    match row {
        Row::Heading(GroupName::Session(name)) => Some(RowKey::Session(name.clone())),
        Row::Window(pane) => Some(RowKey::Window(pane.window_id)),
        Row::Session(c) => Some(RowKey::Claude(c.record.session_id.clone())),
        Row::Heading(GroupName::Outside) | Row::Detail(_) | Row::Empty(_) | Row::Gap => None,
    }
}

/// The selectable row a line belongs to: a detail line belongs to its session, the placeholder to its
/// heading.
pub fn owner(row: &Row) -> Option<RowKey> {
    match row {
        Row::Detail(c) => Some(RowKey::Claude(c.record.session_id.clone())),
        Row::Empty(name) => Some(RowKey::Session(name.clone())),
        _ => key(row),
    }
}

/// Selectable rows in display order.
pub fn keys(rows: &[Row]) -> Vec<RowKey> {
    rows.iter().filter_map(key).collect()
}

/// The row that stands for a pane: its own Claude session, else a row of its window (a Claude session or
/// the window itself), else its tmux session.
pub fn key_for_pane(rows: &[Row], pane: &Pane) -> RowKey {
    let own = rows.iter().find_map(|r| match r {
        Row::Session(c) if c.record.pane_id() == Some(pane.id) => key(r),
        _ => None,
    });
    let in_window = rows.iter().find_map(|r| match r {
        Row::Session(c) if c.pane.as_ref().map(|p| p.window_id) == Some(pane.window_id) => key(r),
        Row::Window(p) if p.window_id == pane.window_id => key(r),
        _ => None,
    });
    own.or(in_window)
        .unwrap_or_else(|| RowKey::Session(pane.session.clone()))
}

/// How many rows scroll off the top to keep `selected` above screen line `end`, with the lines that
/// belong to it when there is room. Derived on every use, so drawing and clicks always agree.
pub fn offset(rows: &[Row], selected: Option<&RowKey>, end: u16) -> usize {
    let capacity = usize::from(end.saturating_sub(FIRST_ROW));
    let lines = || {
        rows.iter()
            .positions(|row| selected.is_some() && owner(row).as_ref() == selected)
    };
    match (lines().next(), lines().next_back()) {
        (Some(first), Some(last)) => (last + 1).saturating_sub(capacity).min(first),
        _ => 0,
    }
}

/// The rows that fit, with the screen line they are drawn on, above screen line `end`.
pub fn placed<'a>(
    rows: &'a [Row],
    selected: Option<&RowKey>,
    end: u16,
) -> impl Iterator<Item = (u16, &'a Row)> {
    (FIRST_ROW..end).zip(rows.iter().skip(offset(rows, selected, end)))
}

/// The selectable row drawn on screen line `y`.
pub fn at(rows: &[Row], selected: Option<&RowKey>, end: u16, y: u16) -> Option<RowKey> {
    placed(rows, selected, end)
        .find(|&(line, _)| line == y)
        .and_then(|(_, row)| owner(row))
}

#[cfg(test)]
mod tests;

//! The marks of the window you are in: a bar in the left column, or a background on its rows.

use ratatui::style::Color;
use ratatui::text::Span;

use crate::listing::GroupName;
use crate::sidebar::mark::ActiveMark;
use crate::sidebar::rows::{self, Row, RowKey};
use crate::sidebar::update::Model;

/// The row belongs to the window you are in: a Claude or window row of that window, or the heading and
/// placeholder of its tmux session while no row of the list stands for that window.
pub fn here(model: &Model, row: &Row) -> bool {
    let window = model.snapshot.window;
    match row {
        Row::Session(c) | Row::Detail(c) => {
            c.pane.as_ref().is_some_and(|p| Some(p.window_id) == window)
        }
        Row::Window(p) => Some(p.window_id) == window,
        Row::Heading(GroupName::Session(s)) | Row::Empty(s) => stands_for_focus(model, s),
        Row::Pad(row) => here(model, row),
        Row::Heading(GroupName::Outside) | Row::Gap => false,
    }
}

/// The focused pane has no Claude or window row, so the heading of its tmux session stands for it.
fn stands_for_focus(model: &Model, session: &str) -> bool {
    model.snapshot.focus.as_ref().is_some_and(|p| {
        rows::key_for_pane(&model.snapshot.rows, p) == RowKey::Session(session.to_owned())
    })
}

/// The cursor's background. Else, in background mode, the background of the window you are in.
pub fn background(model: &Model, here: bool, selected: bool) -> Option<Color> {
    if selected {
        Some(model.theme.cursor)
    } else if here && model.mark == ActiveMark::Background {
        Some(model.theme.active)
    } else {
        None
    }
}

/// The left column of the bar mark: `▌` on the rows of the window you are in, in the text color when a
/// heading (or its `none` placeholder) stands in for that window.
pub fn marker(model: &Model, row: &Row, here: bool) -> Span<'static> {
    match row {
        _ if !here => Span::raw(" "),
        Row::Heading(_) | Row::Empty(_) => Span::raw("▌"),
        Row::Pad(row) => marker(model, row, here),
        _ => Span::styled("▌", model.theme.accent),
    }
}

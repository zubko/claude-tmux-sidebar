//! A window without Claude (`@cts_all_windows`), and how a window is named on a row.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use super::session::bold_if;
use crate::sidebar::update::Model;
use crate::text::truncate;
use crate::tmux::pane::Pane;

/// `  3:docs`: lined up with the names of the Claude rows, bold in the window you are in.
pub fn line(model: &Model, pane: &Pane, width: usize, selected: bool) -> Line<'static> {
    let here = Some(pane.window_id) == model.snapshot.window;
    let name = truncate(&label(pane, &pane.window_name), width.saturating_sub(3));
    let style = Style::new()
        .fg(model.theme.muted)
        .add_modifier(bold_if(here));
    // With the cursor on the window you are in, the background is the cursor's, so the number tells.
    let name = if here && selected {
        number_first(&name, pane.window_index, style, model.theme.accent)
    } else {
        vec![Span::styled(name, style)]
    };
    Line::from([vec![Span::raw("  ")], name].concat())
}

/// `2:` of `2:api` in the `accent` color over `style`, and the rest in `style`; all of it in `accent`
/// when it does not start with the window number.
pub fn number_first(
    name: &str,
    window_index: u32,
    style: Style,
    accent: Color,
) -> Vec<Span<'static>> {
    let number = format!("{window_index}:");
    match name.strip_prefix(&number) {
        Some(rest) => vec![
            Span::styled(number, style.fg(accent)),
            Span::styled(rest.to_owned(), style),
        ],
        None => vec![Span::styled(name.to_owned(), style.fg(accent))],
    }
}

/// `index:name` like the tmux status line, or just the name when the session has one window and the
/// name is not empty.
pub fn label(pane: &Pane, name: &str) -> String {
    if pane.session_windows > 1 || name.is_empty() {
        format!("{}:{name}", pane.window_index)
    } else {
        name.to_owned()
    }
}

//! Rendering: the model becomes lines of styled text. Only [`render`] touches the terminal frame.

use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::rows::{self, Row, RowKey};
use super::theme::Theme;
use super::update::Model;
use crate::i18n::Messages;
use crate::listing::GroupName;
use crate::session::Status;

mod session;
mod window;

const TITLE: &str = "Claude";
/// Statuses counted in the title bar, in order.
const COUNTED: [Status; 3] = [Status::Running, Status::Waiting, Status::Done];

pub fn render(model: &Model, messages: &Messages, frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(
        Paragraph::new(lines(model, messages, area.width, area.height)),
        area,
    );
}

/// The whole screen, one line per terminal row.
pub fn lines(model: &Model, messages: &Messages, width: u16, height: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let end = model.list_end(height);
    let theme = &model.theme;
    let body: Vec<Line<'static>> = if model.snapshot.rows.is_empty() {
        vec![Line::styled(
            format!(" {}", messages.no_sessions),
            theme.muted,
        )]
    } else {
        rows::placed(&model.snapshot.rows, model.selection.selected.as_ref(), end)
            .map(|(_, row)| row_line(model, messages, row, width))
            .collect()
    };
    let filler = usize::from(end.saturating_sub(rows::FIRST_ROW)).saturating_sub(body.len());
    [title(model), Line::styled("─".repeat(width), theme.dim)]
        .into_iter()
        .chain(body)
        .chain(std::iter::repeat_n(Line::default(), filler))
        .chain(model.help.then(|| help(messages, theme)))
        .collect()
}

/// `⏎ jump  j/k move  q close`: bold keys, muted labels.
fn help(messages: &Messages, theme: &Theme) -> Line<'static> {
    let spans = messages
        .help
        .iter()
        .enumerate()
        .flat_map(|(i, (key, label))| {
            let gap = if i == 0 { "" } else { "  " };
            [
                Span::raw(gap),
                Span::styled(*key, Modifier::BOLD),
                Span::raw(" "),
                Span::styled(*label, theme.muted),
            ]
        });
    Line::from(spans.collect::<Vec<_>>())
}

fn title(model: &Model) -> Line<'static> {
    let count = |status: Status| {
        model
            .snapshot
            .rows
            .iter()
            .filter(|r| matches!(r, Row::Session(c) if c.record.state == status))
            .count()
    };
    let counts = COUNTED.into_iter().filter_map(|status| {
        let (icon, color) = session::look(&model.theme, status);
        Some(count(status))
            .filter(|&n| n > 0)
            .map(|n| Span::styled(format!("{icon}{n} "), color))
    });
    let title = Span::styled(
        TITLE,
        Style::new()
            .fg(model.theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    Line::from(
        [title, Span::raw(" ")]
            .into_iter()
            .chain(counts)
            .collect::<Vec<_>>(),
    )
}

fn row_line(model: &Model, messages: &Messages, row: &Row, width: usize) -> Line<'static> {
    let selected = rows::owner(row).is_some_and(|k| model.selection.selected.as_ref() == Some(&k));
    let here = here(model, row);
    let line = match row {
        Row::Gap => Line::default(),
        Row::Empty(_) => empty(model, messages),
        Row::Heading(name) => heading(model, name, messages, here && selected),
        Row::Session(claude) => session::line(model, claude, width, selected),
        Row::Detail(claude) => session::detail(model, claude, width),
        Row::Window(pane) => window::line(model, pane, width, selected),
    };
    match background(&model.theme, here, selected) {
        Some(color) => fill(line, width, color),
        None => line,
    }
}

/// The row belongs to the window you are in: a Claude or window row of that window, or the heading and
/// placeholder of its tmux session while no row of the list stands for that window.
fn here(model: &Model, row: &Row) -> bool {
    let window = model.snapshot.window;
    match row {
        Row::Session(c) | Row::Detail(c) => {
            c.pane.as_ref().is_some_and(|p| Some(p.window_id) == window)
        }
        Row::Window(p) => Some(p.window_id) == window,
        Row::Heading(GroupName::Session(s)) | Row::Empty(s) => stands_for_focus(model, s),
        Row::Heading(GroupName::Outside) | Row::Gap => false,
    }
}

/// The focused pane has no Claude or window row, so the heading of its tmux session stands for it.
fn stands_for_focus(model: &Model, session: &str) -> bool {
    model.snapshot.focus.as_ref().is_some_and(|p| {
        rows::key_for_pane(&model.snapshot.rows, p) == RowKey::Session(session.to_owned())
    })
}

/// The cursor's background, or else the one that marks the rows of the window you are in.
fn background(theme: &Theme, here: bool, selected: bool) -> Option<Color> {
    if selected {
        Some(theme.cursor)
    } else if here {
        Some(theme.active)
    } else {
        None
    }
}

/// `none` where a Claude row's name would start.
fn empty(model: &Model, messages: &Messages) -> Line<'static> {
    Line::styled(
        format!("  {}", messages.none),
        Style::new()
            .fg(model.theme.muted)
            .add_modifier(Modifier::ITALIC),
    )
}

/// `▾ work`; in the accent color when the cursor sits on it while it stands for the window you are in,
/// since the cursor's background hides the one that marks that window.
fn heading(model: &Model, name: &GroupName, messages: &Messages, accented: bool) -> Line<'static> {
    let text = match name {
        GroupName::Session(s) => s.as_str(),
        GroupName::Outside => messages.outside,
    };
    let style = Style::new().add_modifier(Modifier::BOLD);
    let style = if accented {
        style.fg(model.theme.accent)
    } else {
        style
    };
    Line::from(Span::styled(format!("▾ {text}"), style))
}

/// A background across the full width, under the row's own colors.
fn fill(line: Line<'static>, width: usize, color: Color) -> Line<'static> {
    let pad = " ".repeat(width.saturating_sub(line.width()));
    let spans = line
        .spans
        .into_iter()
        .chain([Span::raw(pad)])
        .map(|span| span.patch_style(Style::new().bg(color)))
        .collect::<Vec<_>>();
    Line::from(spans)
}

#[cfg(test)]
mod tests;

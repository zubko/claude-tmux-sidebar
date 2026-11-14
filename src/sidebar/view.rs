//! Rendering: the model becomes lines of styled text. Only [`render`] touches the terminal frame.

use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::compact;
use super::mark::ActiveMark;
use super::rows::{self, Row};
use super::update::Model;
use crate::i18n::Messages;
use crate::listing::GroupName;
use crate::session::Status;
use crate::text;

mod active;
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
    let shown = compact::shown(model, end);
    let body: Vec<Line<'static>> = if shown.is_empty() {
        vec![Line::styled(
            format!(" {}", messages.no_sessions),
            theme.muted,
        )]
    } else {
        rows::placed(&shown, model.selection.selected.as_ref(), end)
            .map(|(_, row)| row_line(model, messages, row, width))
            .collect()
    };
    let filler = usize::from(end.saturating_sub(rows::FIRST_ROW)).saturating_sub(body.len());
    [title(model), Line::styled("─".repeat(width), theme.dim)]
        .into_iter()
        .chain(body)
        .chain(std::iter::repeat_n(Line::default(), filler))
        .chain(model.help.then(|| help(model, messages)))
        .collect()
}

/// `⏎ jump  j/k move  q close`: bold keys, muted labels, after the bar's column when there is one.
fn help(model: &Model, messages: &Messages) -> Line<'static> {
    let spans = messages
        .help
        .iter()
        .enumerate()
        .flat_map(|(i, (key, label))| {
            let gap = if i == 0 { model.mark.margin() } else { "  " };
            [
                Span::raw(gap),
                Span::styled(*key, Modifier::BOLD),
                Span::raw(" "),
                Span::styled(*label, model.theme.muted),
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
        [Span::raw(model.mark.margin()), title, Span::raw(" ")]
            .into_iter()
            .chain(counts)
            .collect::<Vec<_>>(),
    )
}

/// One row. The row drawers learn about the cursor only with the background mark: there the cursor's
/// background hides the mark of the window you are in, so the window index turns accent instead. The bar
/// stays visible under the cursor.
fn row_line(model: &Model, messages: &Messages, row: &Row, width: usize) -> Line<'static> {
    let selected = rows::owner(row).is_some_and(|k| model.selection.selected.as_ref() == Some(&k));
    let here = active::here(model, row);
    let cursor = selected && model.mark == ActiveMark::Background;
    let inner = width.saturating_sub(text::width(model.mark.margin()));
    let line = match row {
        Row::Gap | Row::Pad(_) => Line::default(),
        Row::Empty(_) => empty(model, messages),
        Row::Heading(name) => heading(model, name, messages, here && cursor),
        Row::Session(claude) => session::line(model, claude, inner, cursor),
        Row::Detail(claude) => session::detail(model, claude, inner),
        Row::Window(pane) => window::line(model, pane, inner, cursor),
    };
    let line = match model.mark {
        ActiveMark::Bar => {
            Line::from([vec![active::marker(model, row, here)], line.spans].concat())
        }
        ActiveMark::Background => line,
    };
    match active::background(model, here, selected) {
        Some(color) => fill(line, width, color),
        None => line,
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

/// `▾ work`, in the accent color with `accent`.
fn heading(model: &Model, name: &GroupName, messages: &Messages, accent: bool) -> Line<'static> {
    let text = match name {
        GroupName::Session(s) => s.as_str(),
        GroupName::Outside => messages.outside,
    };
    let style = Style::new().add_modifier(Modifier::BOLD);
    let style = if accent {
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

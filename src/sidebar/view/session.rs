//! One Claude session: status icon, window or directory, and the tool and age on the right; below it
//! the last prompt and context usage.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::window::{self, number_first};
use crate::session::Status;
use crate::session::context_window::ContextUse;
use crate::session::live::Claude;
use crate::sidebar::theme::Theme;
use crate::sidebar::update::Model;
use crate::text::{self, age, truncate};

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
/// The name column keeps at least this many cells when the status text is long.
const MIN_NAME_WIDTH: usize = 12;
/// Context usage from this percent on is a warning, and from the next one on an alarm.
const CONTEXT_WARN: u8 = 50;
const CONTEXT_ALARM: u8 = 80;

pub fn look(theme: &Theme, status: Status) -> (&'static str, Color) {
    match status {
        Status::Waiting => ("◆", theme.waiting),
        Status::Running => ("●", theme.running),
        Status::Done => ("✓", theme.done),
        Status::Idle => ("○", theme.dim),
    }
}

/// `● 2:api                             Bash 3s `.
pub fn line(model: &Model, claude: &Claude, width: usize, selected: bool) -> Line<'static> {
    let (_, color) = look(&model.theme, claude.record.state);
    let right = truncate(
        &status_text(model, claude),
        width.saturating_sub(MIN_NAME_WIDTH),
    );
    let room = width.saturating_sub(4 + text::width(&right));
    let name = truncate(&name(claude), room);
    let used = text::width(&name) + text::width(&right);
    let start = vec![icon(model, claude.record.state), Span::raw(" ")];
    let end = vec![
        Span::raw(" ".repeat(width.saturating_sub(3 + used))),
        Span::styled(right, color),
        Span::raw(" "),
    ];
    Line::from([start, name_spans(model, claude, &name, selected), end].concat())
}

/// `└ fix the failing test…                42% `: the prompt lines up with the name above it.
pub fn detail(model: &Model, claude: &Claude, width: usize) -> Line<'static> {
    let record = &claude.record;
    let theme = &model.theme;
    let windows = &model.snapshot.context_windows;
    let (context, color) = match claude.context().map(|c| windows.resolve(c)) {
        Some(ContextUse::Percent(p)) => (format!("{p}%"), context_color(theme, p)),
        Some(ContextUse::Tokens(n)) => (text::tokens(n), theme.muted),
        None => (String::new(), theme.muted),
    };
    let reserved = if context.is_empty() {
        0
    } else {
        text::width(&context) + 1
    };
    let prompt = truncate(&record.prompt, width.saturating_sub(3 + reserved));
    let gap = width.saturating_sub(3 + text::width(&prompt) + text::width(&context));
    Line::from(vec![
        Span::styled("└ ", theme.dim),
        Span::styled(prompt, theme.muted),
        Span::raw(" ".repeat(gap)),
        Span::styled(context, color),
        Span::raw(" "),
    ])
}

fn context_color(theme: &Theme, percent: u8) -> Color {
    if percent >= CONTEXT_ALARM {
        theme.critical
    } else if percent >= CONTEXT_WARN {
        theme.warning
    } else {
        theme.muted
    }
}

/// The name, bold in the window you are in. With the cursor there too, the background is the cursor's,
/// so the window number (or the whole name when it shows none) takes the accent color instead.
fn name_spans(model: &Model, claude: &Claude, name: &str, selected: bool) -> Vec<Span<'static>> {
    let here = here(model, claude);
    let style = Style::new().add_modifier(bold_if(here));
    match claude.pane.as_ref().filter(|_| here && selected) {
        Some(pane) => number_first(name, pane.window_index, style, model.theme.accent),
        None => vec![Span::styled(name.to_owned(), style)],
    }
}

/// The session runs in the window the sidebar belongs to.
fn here(model: &Model, claude: &Claude) -> bool {
    claude
        .pane
        .as_ref()
        .is_some_and(|p| Some(p.window_id) == model.snapshot.window)
}

/// The status icon; a running session spins.
fn icon(model: &Model, status: Status) -> Span<'static> {
    let (icon, color) = look(&model.theme, status);
    let spinner = SPINNER
        .get(model.frame % SPINNER.len())
        .filter(|_| status == Status::Running);
    let style = Style::new()
        .fg(color)
        .add_modifier(bold_if(status == Status::Waiting));
    Span::styled(*spinner.unwrap_or(&icon), style)
}

pub fn bold_if(on: bool) -> Modifier {
    if on {
        Modifier::BOLD
    } else {
        Modifier::empty()
    }
}

/// The window as the user named it; the directory when tmux named the window itself (after the command
/// it runs, which says nothing about the session), and outside tmux.
fn name(claude: &Claude) -> String {
    match &claude.pane {
        Some(pane) if pane.window_named => window::label(pane, &pane.window_name),
        Some(pane) => window::label(pane, &claude.folder()),
        None => claude.folder().into_owned(),
    }
}

/// `[detail ]age`; the icon already shows the status, and the detail (tool name, or the shell a done
/// session still runs) is not shown while idle.
fn status_text(model: &Model, claude: &Claude) -> String {
    let record = &claude.record;
    let since = record.since.unwrap_or(record.updated_at);
    let base = age(model.now.saturating_sub(since));
    let shows_detail = record.state != Status::Idle && !record.detail.is_empty();
    if shows_detail {
        format!("{} {base}", record.detail)
    } else {
        base
    }
}

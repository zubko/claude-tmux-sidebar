use super::*;
use crate::i18n::{EN, JA};
use crate::listing::Group;
use crate::session::Record;
use crate::session::context_window::ContextWindows;
use crate::session::live::Claude;
use crate::sidebar::rows::RowKey;
use crate::sidebar::selection::Selection;
use crate::sidebar::snapshot::Snapshot;
use crate::text;
use crate::tmux::pane::Pane;
use crate::tmux::pane::fixture::pane;

const WIDTH: u16 = 34;
const CURSOR: Color = Theme::DEFAULT.cursor;
const ACTIVE: Color = Theme::DEFAULT.active;
const ACCENT: Color = Theme::DEFAULT.accent;

fn record(state: Status, detail: &str) -> Record {
    Record {
        session_id: "a".to_owned(),
        state,
        detail: detail.to_owned(),
        cwd: "/src/api".to_owned(),
        since: Some(100),
        ..Record::default()
    }
}

/// Claude `a` in window 2 of a session with `windows` windows; the window is named `api` by hand.
fn claude_in(windows: u32) -> Claude {
    Claude {
        record: record(Status::Done, ""),
        pane: Some(Pane {
            window_name: "api".to_owned(),
            session_windows: windows,
            ..pane(1, 2)
        }),
        latest: None,
    }
}

fn model(state: Status, detail: &str, selected: bool) -> Model {
    let claude = Claude {
        record: record(state, detail),
        ..claude_in(1)
    };
    model_of(claude, selected)
}

fn model_of(claude: Claude, selected: bool) -> Model {
    let groups = [Group {
        name: GroupName::Session("work".to_owned()),
        claudes: vec![claude],
        windows: vec![],
    }];
    let snapshot = Snapshot {
        rows: rows::rows(&groups),
        context_windows: ContextWindows::parse("m=100"),
        ..Snapshot::default()
    };
    let selection = Selection {
        selected: selected.then(|| RowKey::Claude("a".to_owned())),
        synced: None,
    };
    Model {
        snapshot,
        selection,
        now: 102,
        height: 8,
        help: true,
        ..Model::default()
    }
}

fn text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

fn screen(model: &Model, messages: &Messages) -> Vec<String> {
    lines(model, messages, WIDTH, model.height)
        .iter()
        .map(text)
        .collect()
}

/// The background of every span on screen row `row`.
fn backgrounds(model: &Model, row: usize) -> Vec<Option<Color>> {
    let lines = lines(model, &EN, WIDTH, model.height);
    let line = lines.get(row).cloned().unwrap_or_default();
    line.spans.iter().map(|s| s.style.bg).collect()
}

#[test]
fn draws_title_list_and_help() {
    let screen = screen(&model(Status::Waiting, "Edit", false), &EN);
    assert_eq!(screen.len(), 8);
    assert_eq!(screen.first().map(String::as_str), Some("Claude ◆1 "));
    assert_eq!(screen.get(2).map(String::as_str), Some("▾ work"));
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some("◆ api                     Edit 2s ")
    );
    assert_eq!(
        screen.last().map(String::as_str),
        Some("⏎ jump  j/k move  q close")
    );
}

#[test]
fn hiding_the_help_gives_its_line_to_the_list() {
    let model = Model {
        help: false,
        ..model(Status::Waiting, "Edit", false)
    };
    let screen = screen(&model, &EN);
    assert_eq!(screen.len(), 8);
    assert!(screen.iter().all(|line| !line.contains("jump")));
    assert_eq!(model.list_end(model.height), 8);
}

#[test]
fn every_session_has_a_detail_line() {
    let screen = screen(&model(Status::Idle, "", false), &EN);
    assert_eq!(screen.get(4).map(|s| s.trim_end()), Some("└"));
    assert_eq!(
        screen.get(4).map(|s| text::width(s)),
        Some(usize::from(WIDTH))
    );
}

#[test]
fn rows_fill_the_width_and_hide_idle_details() {
    let row = screen(&model(Status::Idle, "Bash", false), &JA)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert_eq!(text::width(&row), usize::from(WIDTH));
    assert!(row.ends_with(" 2s "), "{row}");
    assert!(!row.contains("Bash"), "{row}");
}

#[test]
fn a_done_row_names_the_shell_it_still_runs() {
    let row = screen(&model(Status::Done, "Shell", false), &EN)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert!(row.ends_with(" Shell 2s "), "{row}");
}

#[test]
fn the_cursor_is_a_background_that_keeps_the_row_colors() {
    let model = model(Status::Done, "", true);
    let line = lines(&model, &EN, WIDTH, model.height)
        .get(3)
        .cloned()
        .unwrap_or_default();
    assert_eq!(line.width(), usize::from(WIDTH));
    assert!(line.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    assert!(line.spans.iter().any(|s| s.style.fg == Some(Color::Cyan)));
}

#[test]
fn the_title_and_status_use_the_configured_colors() {
    let theme = Theme {
        accent: Color::Magenta,
        done: Color::Indexed(82),
        ..Theme::default()
    };
    let model = Model {
        theme,
        ..model(Status::Done, "", false)
    };
    let lines = lines(&model, &EN, WIDTH, model.height);
    let colors = |i: usize| -> Vec<Option<Color>> {
        lines
            .get(i)
            .map(|l| l.spans.iter().map(|s| s.style.fg).collect())
            .unwrap_or_default()
    };
    assert!(colors(0).contains(&Some(theme.accent)));
    assert!(colors(3).contains(&Some(theme.done)));
}

mod active;
mod session;
mod windows;

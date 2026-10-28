use super::*;
use crate::session::transcript::Latest;

fn busy_claude(windows: u32, window_name: &str, context: Option<u8>) -> Claude {
    Claude {
        record: Record {
            prompt: "fix the failing sidebar test please".to_owned(),
            ..record(Status::Done, "")
        },
        pane: Some(Pane {
            session_windows: windows,
            window_name: window_name.to_owned(),
            ..pane(1, 2)
        }),
        latest: Some(Latest { context }),
    }
}

#[test]
fn a_session_shows_its_window_then_its_prompt_and_context() {
    let screen = screen(
        &model_of(busy_claude(1, "run the app", Some(42)), false),
        &EN,
    );
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ✓ run the app                 2s ")
    );
    assert_eq!(
        screen.get(4).map(String::as_str),
        Some(" └ fix the failing sidebar t… 42% ")
    );
}

#[test]
fn the_window_index_shows_only_with_several_windows() {
    let screen = screen(&model_of(busy_claude(3, "run the app", None), false), &EN);
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ✓ 2:run the app               2s ")
    );
    assert_eq!(
        screen.get(4).map(String::as_str),
        Some(" └ fix the failing sidebar test … ")
    );
}

#[test]
fn a_window_without_a_name_shows_its_index_like_tmux() {
    let screen = screen(&model_of(busy_claude(1, "", None), false), &EN);
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ✓ 2:                          2s ")
    );
}

#[test]
fn a_narrow_row_cuts_the_window_name() {
    let model = model_of(busy_claude(1, "run the app", Some(90)), true);
    let row = |width| lines(&model, &EN, width, model.height).get(3).map(text);
    assert_eq!(row(16).as_deref(), Some(" ✓ run the … 2s "));
    let lines = lines(&model, &EN, 16, model.height);
    let detail = lines.get(4).cloned().unwrap_or_default();
    assert!(detail.spans.iter().all(|s| s.style.bg == Some(CURSOR)));
    assert!(detail.spans.iter().any(|s| s.style.fg == Some(Color::Red)));
}

//! Decisions about adding and removing the sidebar pane, and the layout saved around it.

use itertools::Itertools;

use crate::text::shell_quote;
use crate::tmux::pane::{Pane, PaneId};

/// Window options that remember the layout from before the sidebar was added.
pub const LAYOUT_OPTION: &str = "@cts_layout";
pub const PANES_OPTION: &str = "@cts_panes";

/// The layout of a window before its sidebar was added, and the panes it described.
#[derive(Debug, PartialEq, Eq)]
pub struct Saved {
    pub layout: String,
    pub panes: Vec<PaneId>,
}

/// The window's own panes, in tmux's order.
pub fn real_panes(panes: &[Pane]) -> Vec<PaneId> {
    panes.iter().filter(|p| !p.sidebar).map(|p| p.id).collect()
}

/// Pane ids as stored in [`PANES_OPTION`].
pub fn encode(ids: &[PaneId]) -> String {
    ids.iter().join(" ")
}

pub fn decode(s: &str) -> Vec<PaneId> {
    s.split_whitespace().filter_map(PaneId::parse).collect()
}

/// A window gets a sidebar unless it has one; a zoomed window would unzoom, so it gets one on the next visit.
pub fn needs_sidebar(panes: &[Pane], zoomed: bool) -> bool {
    !zoomed && !panes.iter().any(|p| p.sidebar)
}

/// The window size a layout was saved at: `236x69` in `bcf0,236x69,0,0,...`, or the root cell's `w` and `h` in
/// the JSON form tmux 3.8 writes.
pub fn size_of(layout: &str) -> Option<String> {
    if layout.starts_with('{') {
        json_size(layout)
    } else {
        classic_size(layout)
    }
}

fn classic_size(layout: &str) -> Option<String> {
    let size = layout.split(',').nth(1)?;
    let (width, height) = size.split_once('x')?;
    let number = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    (number(width) && number(height)).then(|| size.to_owned())
}

fn json_size(layout: &str) -> Option<String> {
    let root: serde_json::Value = serde_json::from_str(layout).ok()?;
    let cell = root.get("L")?;
    let (width, height) = (cell.get("w")?.as_u64()?, cell.get("h")?.as_u64()?);
    Some(format!("{width}x{height}"))
}

/// The layout to restore after the sidebar is gone, and the window size it was saved at: only while the window
/// still has the panes it was saved with, and only when the size can be read. tmux applies a layout at the size
/// written in it and may keep the window size, so the layout must be applied only while the window still has
/// that size.
pub fn restorable<'a>(saved: Option<&'a Saved>, current: &[Pane]) -> Option<(&'a str, String)> {
    let saved = saved.filter(|s| s.panes == real_panes(current))?;
    Some((saved.layout.as_str(), size_of(&saved.layout)?))
}

/// The `if-shell -F` condition and command that apply `layout` to the window `target` only while it is still `size`.
/// The check runs inside tmux, right before `select-layout`, so a resize cannot land between the two.
pub fn guarded_restore(target: &str, layout: &str, size: &str) -> (String, String) {
    (
        format!("#{{==:#{{window_width}}x#{{window_height}},{size}}}"),
        format!("select-layout -t {target} {}", shell_quote(layout)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::pane::fixture::pane;

    fn saved(ids: &str) -> Saved {
        Saved {
            layout: "abcd,80x24,0,0".to_owned(),
            panes: decode(ids),
        }
    }

    #[test]
    fn pane_lists_round_trip_and_read_older_formats() {
        let ids = decode("%1 %2 ");
        assert_eq!(encode(&ids), "%1 %2");
    }

    #[test]
    fn restores_only_with_the_same_panes() {
        let current = [
            pane(1, 0),
            Pane {
                sidebar: true,
                ..pane(5, 0)
            },
            pane(2, 0),
        ];
        assert_eq!(
            restorable(Some(&saved("%1 %2")), &current),
            Some(("abcd,80x24,0,0", "80x24".to_owned()))
        );
        assert_eq!(restorable(Some(&saved("%1 %2 %3")), &current), None);
        assert_eq!(restorable(None, &current), None);
    }

    #[test]
    fn restores_only_a_layout_with_a_readable_size() {
        let unreadable = Saved {
            layout: "garbage".to_owned(),
            panes: decode("%1"),
        };
        assert_eq!(restorable(Some(&unreadable), &[pane(1, 0)]), None);
    }

    #[test]
    fn reads_the_size_of_a_layout() {
        let classic = size_of("bcf0,236x69,0,0{34x69,0,0,359,201x69,35,0,5}");
        assert_eq!(classic.as_deref(), Some("236x69"));
        assert_eq!(size_of("b45d,236x70,0,0,0").as_deref(), Some("236x70"));
        let json = size_of(r#"{"V":2,"L":{"t":"h","w":236,"h":70,"x":0,"y":0,"c":[]}}"#);
        assert_eq!(json.as_deref(), Some("236x70"));
        assert_eq!(size_of(r#"{"V":2}"#), None);
        assert_eq!(size_of("b45d,236x,0,0,0"), None);
        assert_eq!(size_of(""), None);
    }

    #[test]
    fn guards_the_restore_with_the_window_size() {
        let (condition, command) = guarded_restore("@3", "b45d,236x70,0,0,0", "236x70");
        assert_eq!(condition, "#{==:#{window_width}x#{window_height},236x70}");
        assert_eq!(command, "select-layout -t @3 'b45d,236x70,0,0,0'");
    }

    #[test]
    fn skips_windows_with_a_sidebar_or_zoomed() {
        let plain = [pane(1, 0)];
        assert!(needs_sidebar(&plain, false));
        assert!(!needs_sidebar(&plain, true));
        assert!(!needs_sidebar(
            &[Pane {
                sidebar: true,
                ..pane(1, 0)
            }],
            false
        ));
    }
}

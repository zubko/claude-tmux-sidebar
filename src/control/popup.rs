//! The list in a popup instead of the sidebar.

use super::width;
use crate::context::Context;
use crate::text::shell_quote;
use crate::tmux::{self, pane::PaneId};

/// The popup is a little wider than the sidebar to fit its border.
const EXTRA_WIDTH: u16 = 6;
const HEIGHT_OPTION: &str = "@cts_popup_height";
const DEFAULT_HEIGHT: &str = "80%";
/// The width when the popup is wider than the terminal, which tmux rejects (a phone).
const FULL_WIDTH: &str = "100%";
/// Fewer lines show almost nothing, and tmux silently drops a popup under 3 lines.
const MIN_LINES: u32 = 10;

/// Shows the list in a popup on `client` instead of the sidebar. Without a client, tmux picks one, which
/// can be another terminal when several are attached.
pub fn popup(ctx: &Context, origin: Option<PaneId>, client: Option<&str>) {
    let origin = origin.map(|o| o.to_string()).unwrap_or_default();
    let command = format!(
        "{} popup-ui {}",
        shell_quote(&ctx.exe.to_string_lossy()),
        shell_quote(&origin)
    );
    let width = (width() + EXTRA_WIDTH).to_string();
    let option = tmux::global_option(HEIGHT_OPTION);
    sizes(&width, option.as_deref())
        .into_iter()
        .any(|(width, height)| open(&command, client, width, height));
}

fn open(command: &str, client: Option<&str>, width: &str, height: &str) -> bool {
    let args: Vec<&str> = ["display-popup", "-E"]
        .into_iter()
        .chain(client.into_iter().flat_map(|c| ["-c", c]))
        .chain(["-w", width, "-h", height, "-T", " Claude ", command])
        .collect();
    tmux::accepted(&args)
}

/// The popup sizes to try in order, as (width, height): every height at `width`, then every height at
/// the full width.
fn sizes<'a>(width: &'a str, option: Option<&'a str>) -> Vec<(&'a str, &'a str)> {
    let heights = heights(option);
    [width, FULL_WIDTH]
        .into_iter()
        .flat_map(|w| heights.iter().map(move |h| (w, *h)))
        .collect()
}

/// The popup heights to try in order: the `@cts_popup_height` value when it is at least 10 lines (`40`)
/// or a percent (`100%`), then the default. tmux still rejects some of these values, such as `101%` or
/// more lines than the terminal has.
fn heights(option: Option<&str>) -> Vec<&str> {
    let digits = |v: &str| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit());
    let valid = |v: &&str| match v.strip_suffix('%') {
        Some(percent) => digits(percent),
        None => digits(v) && v.parse::<u32>().is_ok_and(|n| n >= MIN_LINES),
    };
    option
        .filter(valid)
        .filter(|v| *v != DEFAULT_HEIGHT)
        .into_iter()
        .chain([DEFAULT_HEIGHT])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tries_10_lines_or_more_or_a_percent_then_80_percent() {
        assert_eq!(heights(Some("100%")), ["100%", "80%"]);
        assert_eq!(heights(Some("40")), ["40", "80%"]);
        assert_eq!(heights(Some("10")), ["10", "80%"]);
        assert_eq!(heights(Some("9")), ["80%"]);
        assert_eq!(heights(Some("0")), ["80%"]);
        assert_eq!(heights(Some("101%")), ["101%", "80%"]);
        assert_eq!(heights(Some("80%")), ["80%"]);
        assert_eq!(heights(None), ["80%"]);
        assert_eq!(heights(Some("%")), ["80%"]);
        assert_eq!(heights(Some("tall")), ["80%"]);
        assert_eq!(heights(Some("-5")), ["80%"]);
    }

    #[test]
    fn tries_the_full_width_after_every_height_at_the_given_width() {
        assert_eq!(
            sizes("52", Some("90%")),
            [
                ("52", "90%"),
                ("52", "80%"),
                ("100%", "90%"),
                ("100%", "80%")
            ]
        );
        assert_eq!(sizes("52", None), [("52", "80%"), ("100%", "80%")]);
    }
}

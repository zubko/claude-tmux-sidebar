//! One line per Claude session instead of two, chosen by the `@cts_compact` and `@cts_auto_compact` tmux
//! options. The snapshot keeps the detail lines; drawing and clicks leave them out, except the selected
//! session's.

use std::borrow::Cow;

use super::rows::{self, Row};
use super::update::Model;

const OPTION: &str = "@cts_compact";
const AUTO_OPTION: &str = "@cts_auto_compact";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Compact {
    /// Every Claude session takes two lines.
    #[default]
    Off,
    /// Every Claude session but the selected one takes one line: its detail line is left out.
    On,
    /// `On` only when the full list does not fit.
    Auto,
}

impl Compact {
    /// The mode for the options; `@cts_compact on` wins over `@cts_auto_compact on`.
    pub fn from_options(option: impl Fn(&str) -> Option<String>) -> Self {
        match (option(OPTION).as_deref(), option(AUTO_OPTION).as_deref()) {
            (Some("on"), _) => Self::On,
            (_, Some("on")) => Self::Auto,
            _ => Self::Off,
        }
    }
}

/// The rows to draw above screen line `end`: when the list is compact, only the selected session keeps its
/// detail line.
pub fn shown(model: &Model, end: u16) -> Cow<'_, [Row]> {
    let all = model.snapshot.rows.as_slice();
    let compact = match model.compact {
        Compact::Off => false,
        Compact::On => true,
        Compact::Auto => all.len() > usize::from(end.saturating_sub(rows::FIRST_ROW)),
    };
    if compact {
        Cow::Owned(
            all.iter()
                .filter(|r| {
                    !matches!(r, Row::Detail(_))
                        || rows::owner(r).as_ref() == model.selection.selected.as_ref()
                })
                .cloned()
                .collect(),
        )
    } else {
        Cow::Borrowed(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::listing::GroupName;
    use crate::session::Record;
    use crate::session::live::Claude;
    use crate::sidebar::snapshot::Snapshot;

    fn options(compact: &str, auto: &str) -> Compact {
        Compact::from_options(|name| match name {
            OPTION => Some(compact.to_owned()),
            AUTO_OPTION => Some(auto.to_owned()),
            _ => None,
        })
    }

    /// A heading, a Claude session and its detail line: three lines in full, two when compact.
    fn model(compact: Compact) -> Model {
        let claude = Claude {
            record: Record::default(),
            pane: None,
            latest: None,
        };
        let rows = vec![
            Row::Heading(GroupName::Outside),
            Row::Session(claude.clone()),
            Row::Detail(claude),
        ];
        Model {
            compact,
            snapshot: Snapshot {
                rows,
                ..Snapshot::default()
            },
            ..Model::default()
        }
    }

    fn lines(compact: Compact, end: u16) -> usize {
        shown(&model(compact), end).len()
    }

    #[test]
    fn compact_wins_over_auto_and_anything_but_on_is_off() {
        assert_eq!(options("on", "on"), Compact::On);
        assert_eq!(options("off", "on"), Compact::Auto);
        assert_eq!(options("off", "yes"), Compact::Off);
        assert_eq!(Compact::from_options(|_| None), Compact::Off);
    }

    #[test]
    fn leaves_out_detail_lines_always_or_when_the_list_does_not_fit() {
        let fits = rows::FIRST_ROW + 3;
        let short = rows::FIRST_ROW + 2;
        assert_eq!(lines(Compact::Off, short), 3);
        assert_eq!(lines(Compact::On, fits), 2);
        assert_eq!(lines(Compact::Auto, fits), 3);
        assert_eq!(lines(Compact::Auto, short), 2);
    }

    #[test]
    fn the_selected_session_keeps_its_detail_line() {
        let mut model = model(Compact::On);
        model.selection.selected = Some(rows::RowKey::Claude(String::new()));
        assert_eq!(shown(&model, rows::FIRST_ROW).len(), 3);
    }
}

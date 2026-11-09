//! How the rows of the window you are in are marked, chosen by the `@cts_active_mark` tmux option.

/// `bar` or `background`.
pub const OPTION: &str = "@cts_active_mark";

/// `Default` is `Background`, upstream's look, so the view tests keep upstream's output. The option's
/// default is `Bar`; see [`ActiveMark::parse`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActiveMark {
    /// A `▌` in a column on the left of every row.
    Bar,
    /// The `@cts_color_active` background on the rows.
    #[default]
    Background,
}

impl ActiveMark {
    /// The mark for the option's value; the bar for anything unset or unknown.
    pub fn parse(option: Option<&str>) -> Self {
        match option {
            Some("background") => Self::Background,
            _ => Self::Bar,
        }
    }

    /// The column that the bar mode adds before every row, the title and the help line.
    pub fn margin(self) -> &'static str {
        match self {
            Self::Bar => " ",
            Self::Background => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_option_defaults_to_the_bar() {
        assert_eq!(ActiveMark::parse(None), ActiveMark::Bar);
        assert_eq!(ActiveMark::parse(Some("bar")), ActiveMark::Bar);
        assert_eq!(
            ActiveMark::parse(Some("background")),
            ActiveMark::Background
        );
        assert_eq!(ActiveMark::parse(Some("nope")), ActiveMark::Bar);
    }

    #[test]
    fn only_the_bar_takes_a_column() {
        assert_eq!(ActiveMark::Bar.margin(), " ");
        assert_eq!(ActiveMark::Background.margin(), "");
    }
}

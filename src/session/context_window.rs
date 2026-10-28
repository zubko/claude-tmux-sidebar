//! Context window sizes by model, as listed in the `@cts_context_windows` tmux option.

use std::collections::HashMap;

/// Context window size in tokens, by model id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextWindows(HashMap<String, u64>);

/// What a session knows about its context: the input tokens of its last request and their model, or
/// the percent a hook before 0.7.4 wrote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context<'a> {
    Tokens { tokens: u64, model: &'a str },
    Percent(u8),
}

/// What a row shows: a share of the window when its size is listed, else the token count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextUse {
    Percent(u8),
    Tokens(u64),
}

impl ContextWindows {
    /// Reads `model=size` pairs separated by spaces, skipping malformed ones.
    pub fn parse(list: &str) -> Self {
        Self(
            list.split_whitespace()
                .filter_map(|pair| {
                    let (model, size) = pair.split_once('=')?;
                    let size = size.parse().ok().filter(|&n| n > 0)?;
                    Some((model.to_owned(), size)).filter(|_| !model.is_empty())
                })
                .collect(),
        )
    }

    /// `context` as a share of its model's window, capped at 100.
    pub fn resolve(&self, context: Context<'_>) -> ContextUse {
        match context {
            Context::Percent(percent) => ContextUse::Percent(percent),
            Context::Tokens { tokens, model } => {
                self.0
                    .get(model)
                    .map_or(ContextUse::Tokens(tokens), |&size| {
                        let share = tokens.saturating_mul(100) / size;
                        ContextUse::Percent(u8::try_from(share.min(100)).unwrap_or(100))
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(tokens: u64, model: &str) -> Context<'_> {
        Context::Tokens { tokens, model }
    }

    #[test]
    fn reads_the_list_and_skips_malformed_pairs() {
        let windows = ContextWindows::parse("a=100 b=x c=0 d =200 e=200");
        assert_eq!(windows.resolve(tokens(50, "a")), ContextUse::Percent(50));
        assert_eq!(windows.resolve(tokens(50, "e")), ContextUse::Percent(25));
        assert_eq!(windows.resolve(tokens(50, "b")), ContextUse::Tokens(50));
        assert_eq!(windows.resolve(tokens(50, "c")), ContextUse::Tokens(50));
        assert_eq!(windows.resolve(tokens(50, "")), ContextUse::Tokens(50));
    }

    #[test]
    fn only_a_listed_model_gets_a_share() {
        let windows = ContextWindows::parse("claude-opus-5-5=200000");
        assert_eq!(
            windows.resolve(tokens(100_000, "claude-opus-5-5")),
            ContextUse::Percent(50)
        );
        assert_eq!(
            windows.resolve(tokens(400_000, "claude-opus-5-5[1m]")),
            ContextUse::Tokens(400_000)
        );
        assert_eq!(
            windows.resolve(tokens(900_000, "claude-opus-5-5")),
            ContextUse::Percent(100)
        );
    }

    #[test]
    fn an_old_percent_passes_through() {
        let windows = ContextWindows::default();
        assert_eq!(
            windows.resolve(Context::Percent(64)),
            ContextUse::Percent(64)
        );
    }
}

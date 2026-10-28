use super::*;

fn assistant(model: &str, branch: &str, tokens: u64, sidechain: bool) -> String {
    format!(
        r#"{{"type":"assistant","isSidechain":{sidechain},"gitBranch":"{branch}","requestedModel":"{model}","message":{{"model":"x","usage":{{"input_tokens":{tokens},"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}}}}"#
    )
}

#[test]
fn picks_the_last_main_thread_assistant_entry() {
    let text = [
        assistant("claude-opus-5-5", "old", 20_000, false),
        assistant("claude-opus-5-5", "main", 100_000, false),
        assistant("claude-haiku-5-5", "side", 10_000, true),
        r#"{"type":"user","message":{"content":"hi"}}"#.to_owned(),
        "partial line".to_owned(),
    ]
    .join("\n");
    assert_eq!(
        latest(&text),
        Some(Latest {
            context: Some(Usage {
                tokens: 100_000,
                model: "claude-opus-5-5".to_owned()
            })
        })
    );
    assert_eq!(latest("nothing"), None);
}

fn boundary(branch: &str, sidechain: bool) -> String {
    format!(
        r#"{{"type":"system","subtype":"compact_boundary","isSidechain":{sidechain},"gitBranch":"{branch}","compactMetadata":{{}}}}"#
    )
}

#[test]
fn a_compaction_after_the_last_assistant_entry_leaves_the_context_unknown() {
    let text = [
        assistant("claude-opus-5-5", "old", 180_000, false),
        boundary("after", false),
        r#"{"type":"user","message":{"content":"summary"}}"#.to_owned(),
    ]
    .join("\n");
    assert_eq!(latest(&text), Some(Latest { context: None }));
}

#[test]
fn an_assistant_entry_after_a_compaction_wins() {
    let text = [
        boundary("old", false),
        assistant("claude-opus-5-5", "new", 20_000, false),
    ]
    .join("\n");
    assert_eq!(
        latest(&text),
        Some(Latest {
            context: Some(Usage {
                tokens: 20_000,
                model: "claude-opus-5-5".to_owned()
            })
        })
    );
}

#[test]
fn only_a_main_thread_boundary_entry_counts_as_a_compaction() {
    let text = [
        assistant("claude-opus-5-5", "main", 100_000, false),
        boundary("side", true),
        r#"{"type":"user","message":{"content":"what is compact_boundary?"}}"#.to_owned(),
    ]
    .join("\n");
    assert_eq!(
        latest(&text),
        Some(Latest {
            context: Some(Usage {
                tokens: 100_000,
                model: "claude-opus-5-5".to_owned()
            })
        })
    );
}

#[test]
fn the_requested_model_wins_over_the_answering_one() {
    let entry =
        r#"{"type":"assistant","message":{"model":"claude-opus-5-5","usage":{"input_tokens":5}}}"#;
    let model = |text: &str| latest(text).and_then(|l| l.context).map(|u| u.model);
    assert_eq!(model(entry), Some("claude-opus-5-5".to_owned()));
    assert_eq!(
        model(&assistant("claude-opus-5-5[1m]", "main", 5, false)),
        Some("claude-opus-5-5[1m]".to_owned())
    );
}

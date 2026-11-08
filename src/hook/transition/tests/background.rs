//! Background tasks that still run when the main thread stops.

use super::*;

fn busy(kind: &'static str) -> Event<'static> {
    Event::Stop { busy: Some(kind) }
}

fn idle() -> Event<'static> {
    Event::Notification {
        kind: "idle_prompt",
        agent: "",
    }
}

#[test]
fn background_subagents_keep_a_stopped_session_running() {
    let working = run(&[Event::UserPromptSubmit, busy("subagent"), idle()]);
    assert_eq!(
        (working.state, working.detail.as_str(), working.background),
        (Status::Running, "Agent", true)
    );
    let done = run(&[busy("subagent"), Event::UserPromptSubmit, DONE]);
    assert_eq!((done.state, done.background), (Status::Done, false));
}

#[test]
fn a_background_shell_leaves_the_session_done_with_its_name() {
    // A subagent hands back, which wakes the main thread; only its shell still runs.
    let shell = run(&[
        busy("subagent"),
        Event::UserPromptSubmit,
        busy("shell"),
        idle(),
    ]);
    assert_eq!(
        (shell.state, shell.detail.as_str(), shell.background),
        (Status::Done, "Shell", false)
    );
}

#[test]
fn idle_prompt_ends_an_interrupted_turn_while_a_shell_runs() {
    let interrupted = run(&[busy("shell"), Event::UserPromptSubmit, idle()]);
    assert_eq!(
        (interrupted.state, interrupted.detail.as_str()),
        (Status::Idle, "")
    );
}

#[test]
fn a_new_process_has_no_background_tasks() {
    let resumed = run(&[
        busy("subagent"),
        Event::SessionStart { source: "resume" },
        idle(),
    ]);
    assert!(!resumed.background);
    let started = run(&[busy("shell"), Event::SessionStart { source: "startup" }]);
    assert_eq!((started.state, started.detail.as_str()), (Status::Done, ""));
}

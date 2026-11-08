use std::io;
use std::os::unix::process::parent_id;

use super::*;
use crate::tmux::pane::fixture::pane;

const SESSION: &str = "b413e892-53af-4c1a-a469-61b25d01b221";

fn claude_json(pid: u32, tmux: &str) -> String {
    format!(
        r#"{{"pid":{pid},"sessionId":"{SESSION}","cwd":"/Users/me/orca/slate-2","startedAt":1700000000500,
            "kind":"interactive","tmux":"{tmux}","name":"slate-2-2f","status":"idle"}}"#
    )
}

/// A Claude directory listing one session, and a context pointing at it and at an empty state dir.
fn setup(dir: &Path, json: &str) -> io::Result<Context> {
    let claude_dir = dir.join("claude");
    fs::create_dir_all(claude_dir.join(SESSIONS_DIR))?;
    fs::write(claude_dir.join(SESSIONS_DIR).join("1.json"), json)?;
    let state_dir = dir.join("state");
    store::ensure_dir(&state_dir)?;
    Ok(Context {
        state_dir,
        pane: None,
        socket: "s".to_owned(),
        exe: PathBuf::new(),
        debug: false,
        claude_dir,
    })
}

/// Pane `%1`, run by `pid`.
fn pane_run_by(pid: u32) -> Pane {
    Pane { pid, ..pane(1, 0) }
}

#[test]
fn reads_what_claude_writes_about_a_session() {
    let known = Known::parse(claude_json(42, "s1:@9.%18").as_bytes());
    assert_eq!(
        known,
        Some(Known {
            pid: 42,
            session_id: SESSION.to_owned(),
            cwd: "/Users/me/orca/slate-2".to_owned(),
            started_at: 1_700_000_000_500,
            tmux: "s1:@9.%18".to_owned(),
        })
    );
    assert_eq!(known.and_then(|k| k.pane()), PaneId::parse("%18"));
    assert_eq!(
        Known::parse(claude_json(42, "").as_bytes()).and_then(|k| k.pane()),
        None
    );
    assert_eq!(Known::parse(br#"{"pid":1,"sessionId":"../x"}"#), None);
}

#[test]
fn names_the_transcript_like_claude_code() {
    assert_eq!(
        transcript(Path::new("/h/.claude"), "/Users/me/orca/slate_2.x", "id"),
        PathBuf::from("/h/.claude/projects/-Users-me-orca-slate-2-x/id.jsonl")
    );
}

#[test]
fn seeds_a_session_running_in_a_pane_of_this_server() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let me = std::process::id();
    let ctx = setup(dir.path(), &claude_json(me, "t:@0.%1"))?;
    let transcript = transcript(&ctx.claude_dir, "/Users/me/orca/slate-2", SESSION);
    fs::create_dir_all(transcript.parent().unwrap_or(dir.path()))?;
    fs::write(&transcript, "")?;
    let seeded = Seeded::default().seed(&ctx, &[pane_run_by(parent_id())]);
    let record = store::read(&ctx.state_dir, SESSION).unwrap_or_default();
    assert_eq!(record.state, Status::Idle);
    assert_eq!(record.pane, "%1");
    assert_eq!(record.socket, "s");
    assert_eq!(record.pid, Some(me));
    assert_eq!(record.cwd, "/Users/me/orca/slate-2");
    assert_eq!(record.transcript, transcript.to_string_lossy());
    assert_eq!(record.since, Some(1_700_000_000));
    assert!(record.updated_at > 1_700_000_000);
    assert!(seeded.0.contains(&(me, SESSION.to_owned())));
    Ok(())
}

#[test]
fn leaves_a_record_a_hook_wrote_alone() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let ctx = setup(dir.path(), &claude_json(std::process::id(), "t:@0.%1"))?;
    let running = Record {
        session_id: SESSION.to_owned(),
        state: Status::Running,
        ..Record::default()
    };
    store::write(&ctx.state_dir, &running)?;
    let seeded = Seeded::default().seed(&ctx, &[pane_run_by(parent_id())]);
    assert_eq!(store::read(&ctx.state_dir, SESSION), Some(running));
    assert!(seeded.0.is_empty());
    Ok(())
}

#[test]
fn skips_sessions_outside_this_server() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let me = std::process::id();
    let ctx = setup(dir.path(), &claude_json(me, "t:@0.%1"))?;
    // Pane %1 is not in this server.
    let seeded = Seeded::default().seed(
        &ctx,
        &[Pane {
            pid: parent_id(),
            ..pane(2, 0)
        }],
    );
    assert!(seeded.0.is_empty() && store::read_all(&ctx.state_dir).is_empty());
    // A pane with the same id in this server, but run by a process this one does not run under.
    let seeded = seeded.seed(&ctx, &[pane_run_by(1)]);
    assert!(store::read_all(&ctx.state_dir).is_empty());
    assert!(seeded.0.contains(&(me, SESSION.to_owned())));
    // Looked at once: the right pane later does not get a record from this sidebar.
    let seeded = seeded.seed(&ctx, &[pane_run_by(parent_id())]);
    assert!(store::read_all(&ctx.state_dir).is_empty());
    assert_eq!(seeded.0.len(), 1);
    Ok(())
}

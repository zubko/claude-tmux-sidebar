# claude-tmux-sidebar (fork)

A tmux plugin in Rust. It puts a sidebar pane on the left of every tmux window. The sidebar lists all tmux sessions
and windows, and the status of the AI agents running in them. Today the agent is Claude Code; its hooks report the
status.

This repo is a fork of `torikatsupg/claude-tmux-sidebar` (git remote `upstream`). [fork] It is published as
`zubko/claude-tmux-sidebar` (remote `origin`). A bullet or sentence that starts with `[fork]` describes something
that is different from upstream; everything else is the same in both.

`README.md` has the user docs (install, keys, options). `CONTRIBUTING.md` has the full hook-to-status table, the
status edge cases and the release steps. This file is the short guide for an agent working on the code.

## Features

- One status icon per Claude session: running (green spinner), waiting for you (yellow `◆`), done (cyan `✓`), idle
  (gray `○`). The tool in use and the time in that status sit on the right.
- A second line per session: the last prompt and the context usage (yellow from 50%, red from 80%).
- `Enter` or a click goes to the session's pane. `n` / `p` in the sidebar and `prefix C-n` / `C-p` from anywhere go to
  the next or previous Claude that is done or waiting.
- The same list in a popup (`prefix B`). Sessions without Claude are listed too, so the sidebar is also a session
  switcher.
- Options are tmux global options with the prefix `@cts_` (`@cts_width`, `@cts_toggle_key`, `@cts_lang`, ...). See
  `README.md` for the list.

## Repo layout

- `src/` is the one binary, `claude-tmux-sidebar`. `tests/hook.rs` runs the real binary end to end.
- `bin/cts` runs the binary (tmux side). `bin/cts-hook` runs `claude-tmux-sidebar hook` (Claude Code side). Both are
  small `sh` wrappers that install the binary first when it is missing.
- `scripts/fetch-binary.sh` installs `bin/claude-tmux-sidebar`: the release binary for this checkout's version
  (SHA-256 checked), or a `cargo build` when no release binary fits. `--build` always builds from source.
- `sidebar.tmux` is the TPM entry point. It binds the keys and sets the tmux hooks (`after-new-window`, ...) at a
  fixed hook index (`@cts_hook_index`, default 77), so re-sourcing is safe.
- `hooks/hooks.json` is the Claude Code plugin's hook list; every event runs `bin/cts-hook`. `.claude-plugin/` holds
  the plugin and marketplace manifests. `install.sh` is the no-TPM installer.
- `xtask/` is `cargo xtask ci`. `deny.toml` is for `cargo deny` (runs in CI only).
- `.github/workflows/ci.yml` runs `cargo xtask ci`, the macOS tests and `cargo deny`. `release.yml` builds and
  publishes a release (see below).

## How it works

1. A Claude Code hook runs `bin/cts-hook`, which runs `claude-tmux-sidebar hook` with the payload on stdin. The hook
   turns the event into a status (`hook::transition`, a pure state machine), merges it into the session's record
   (`hook::compose`) and writes `~/.cache/claude-tmux-sidebar/<session_id>.json`. `SessionEnd` removes the file.
   A hook never fails Claude: errors are swallowed and the wrapper skips the event while the binary is missing.
2. tmux hooks run `cts ensure` / `cts fix-width`: they add a sidebar pane to a window when the sidebar is enabled
   (`@cts_enabled`), and restore its width after layout changes. The sidebar pane runs `claude-tmux-sidebar sidebar`.
   The window's layout from before the sidebar is saved in window options and restored on close (`control::layout`).
3. The sidebar loop (`sidebar::run`) reloads a `Snapshot` every 500 ms or as soon as the state directory changes:
   `tmux list-panes -a`, the state files, and the tail of each session's transcript (context usage). It drops state
   files whose pane or process is gone, and closes itself when its window has no other pane. Each step reads input,
   applies the pure `update(model, msg) -> (model, effects)` and runs the effects (`effect`).
   [fork] Before it reads the state files, `session::seed` writes an idle record for every Claude that Claude Code
   lists in `~/.claude/sessions/<pid>.json` with a pane of this server but no state file (its `SessionStart` hook
   was dropped, which can happen when many sessions start at once after a reboot). The record is written only when
   the file is still missing, and each session is checked once per sidebar process.
4. `next` / `prev` (`jump`) and the popup (`popup` / `popup-ui`) use the same loaders without a sidebar pane.

Statuses in short (the full table is in `CONTRIBUTING.md`): prompt and tool events mean running; `PermissionRequest`,
`Elicitation`, `AskUserQuestion` and the matching notifications mean waiting; `Stop` with nothing left in the
background means done. A new session starts idle; `SessionStart` keeps the status the session already had.
`idle_prompt` turns only a running or waiting session idle, and does nothing while background work runs. Subagent
events carry the parent's `session_id`; only the agent that raised a prompt ends the wait.
[fork] A background shell does not count as background work: a dev server may never end. A `Stop` that leaves only
shells running means done with the detail `Shell`, and `idle_prompt` keeps that detail.

Environment: `TMUX_PANE` and `TMUX` (socket) say where the process runs; `CTS_STATE_DIR` overrides the state
directory (tests use it); `CTS_DEBUG=1` appends raw hook payloads to `debug.log` in the state directory.
[fork] `CLAUDE_CONFIG_DIR` (default `~/.claude`) is where the sidebar reads Claude Code's own session list.

## Module map (`src/`)

Pure modules turn values into values. Side effects stay at the edges: `tmux`, `session::store`, `process`, and the
`run` functions of `control`, `hook`, `jump` and `sidebar`.

| Module | Role |
|---|---|
| `cli`, `lib`, `main` | argument parsing (`USAGE` lists every command) and dispatch |
| `context` | the process environment, read once |
| `tmux`, `tmux::pane` | run tmux; parse `list-panes -F` into `Pane` (`PaneId` `%n`, `WindowId` `@n`) |
| `session` | `Record` (the state file format), `Status`; `store` reads and writes the files |
| `session::live` | which state files are live sessions of this server (`Claude` = record + pane + transcript) |
| `session::seed` | [fork] a record for a Claude that Claude Code lists in a pane of this server but no hook recorded |
| `session::transcript` | the context tokens and model from the transcript tail; `cache` re-reads only changed files |
| `session::context_window` | [fork] `@cts_context_windows` parsing; tokens to a percent of the window |
| `process` | is a pid alive; which Claude process started this hook |
| `hook` | `event` (payload), `transition` (status state machine), `compose` (next record) |
| `control`, `control::layout` | open / close / ensure / fix-width; the saved layout |
| `listing` | groups: tmux sessions in tmux order, then "outside tmux"; [fork] windows without Claude |
| `jump` | the next / previous done or waiting Claude |
| `sidebar` | the loop; `snapshot` (load), `rows`, `selection`, `update` (pure), `view` (render), `input`, `effect` |
| `sidebar::update::destination` | [fork] where a row goes: a Claude pane, a window's pane, or a session |
| `text`, `i18n` | width, truncation, ages, token counts, shell quoting; `en` / `ja` strings |

Tests sit in `mod tests` at the bottom of a file, or in `<module>/tests.rs` and `<module>/tests/*.rs` when they
grow. Test fixtures: `tmux::pane::fixture::pane`, `CTS_STATE_DIR` with a `tempfile` dir.

## How the list looks

- One heading per tmux session, in tmux order. Under it, its windows with Claude in window order ([fork] every window
  with `@cts_show_windows on`). Sessions outside tmux come last under one "(outside tmux)" heading. [fork] Rows
  outside tmux sort by their folder name, then session id.
- A window with Claude shows the Claude row(s): status icon, window label, tool and age, and a second line with the
  last prompt and the context usage.
- [fork] With `@cts_show_windows on` (default off), a window without Claude is a dim one-line row (`Row::Window`).
  The cursor stops on it. `Enter` or a click goes to the pane the user works in there (`tmux::pane::focused`), never
  to a sidebar pane. `n` / `p` and `prefix C-n` / `C-p` visit only Claude rows that are done or waiting.
- [fork] A tmux session heading is a row too (`RowKey::Session`): the cursor stops on it, and `Enter` or a click
  switches to that session. The `none` placeholder under a session with nothing listed belongs to its heading.
  The row that stands for a pane (`rows::key_for_pane`) is its own Claude row, else a row of its window, else
  its session heading. So when the window you are in has no row, its heading gets the cursor and a bar in the
  text color (not coral).
- [fork] The window label is what the tmux status line shows: `#I:#W`. The index is left out only when the session has
  one window and the name is not empty. The directory shows only outside tmux. No git branch.
- [fork] Context usage is a share of the window size listed in the `@cts_context_windows` tmux option
  (`model=tokens ...`). The model id is the transcript entry's `requestedModel`, or `message.model` when that is
  empty; the tokens are input plus cache creation plus cache read. Only an exact model id gets a percentage; an
  unlisted or empty model shows the token count (`61k`). Never hardcode window sizes or guess them from the model
  name. The real size for an account comes from `claude -p --model <m> --output-format json "hi"` →
  `modelUsage.<id>.contextWindow` (a JSON array when `verbose` is on, one object otherwise).
- [fork] The accent color (title, current-window bar) is Claude's coral, `#D97757` (`ACCENT` in `sidebar::view`).
- Keys in the list: `j` / `k` / arrows move, `g` / `G` first / last, `n` / `p` next / previous done or waiting Claude,
  `Enter` / `l` / click go, `r` reload, `q` close (`Esc` too in the popup).

## Commands

- `cargo xtask ci` before a commit: rustfmt, clippy, tests, shellcheck, shfmt and the module size check. It needs
  `shellcheck` and `shfmt`; when they are missing, run `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --locked`, and say
  that the shell checks did not run.
- `scripts/fetch-binary.sh --build` builds `bin/claude-tmux-sidebar` from this checkout for a live tmux.
- `scripts/screenshot.sh` regenerates `docs/screenshot.svg`; `scripts/demo.sh` re-records `docs/demo.gif`.
- The Rust toolchain is pinned in `rust-toolchain.toml`.

## Style rules

- Clippy is strict (`Cargo.toml` `[workspace.lints]`, `clippy.toml`): `pedantic` on, no `unwrap` / `expect` /
  `panic` / indexing / `as` casts, no `unsafe`, functions of at most 30 lines, at most 4 arguments.
- Every Rust file stays at 200 lines or less, tests included (`cargo xtask ci` checks it). Split tests into
  `<module>/tests/*.rs` when a file grows.
- Iterator adapters (`map`, `filter`, `fold`) over mutable loops. Keep side effects at the edges; new logic goes in a
  pure function with a test.
- Doc comments on modules and public items, in the same short plain English as the existing ones.
- Shell scripts pass `shellcheck` and `shfmt`.

## Testing rules

- Never run tests or experiments against the real tmux server. Use a throwaway server with an explicit socket
  (`tmux -S <dir>/tmux.sock ...`) and kill only that socket.
- A tmux client without a UTF-8 locale prints tabs in `-F` output as `_`, and the pane list then parses as empty.
  Set `LANG=en_US.UTF-8` when you run the binary or tmux from a shell.
- `tests/hook.rs` runs the built binary with `CTS_STATE_DIR` set to a temp dir and `TMUX` unset.

## Trying a build in the live sidebar

Build with `cargo build --release --locked`, keep a backup of the installed binary, move the new one over
`~/.tmux/plugins/claude-tmux-sidebar/bin/claude-tmux-sidebar`, then run `bin/cts off` and `bin/cts on` to restart
every sidebar. Ask the user first: this changes their running tmux.

## Releases and the fork

- [fork] The version has a fork suffix (`0.7.5-zubko.2`) in `Cargo.toml`, `Cargo.lock`, `.claude-plugin/plugin.json`,
  `.claude-plugin/marketplace.json` and `CHANGELOG.md`; the release workflow checks that they match and that the tag
  is new.
- [fork] The README install lines, the manifest links and the plugin marketplace name (`zubko`) point at this fork.
- [fork] `scripts/fetch-binary.sh` downloads from this fork (`REPO_URL`). Keep it that way, so a machine never runs an
  upstream binary with this fork's scripts.
- [fork] The workflows use only GitHub's own actions, pinned to a full commit SHA; Rust comes from the runner's
  `rustup` (`rustup toolchain install`), not from a third-party action.
- [fork] The build job must not get `id-token: write`; the provenance attestation runs in its own `attest` job that
  builds nothing. Every checkout uses `persist-credentials: false`; `gh release create --target` makes the tag.
- Releases are immutable. Only the repository owner can run the Release workflow, and only on `main`.
- Release steps: bump the version everywhere, add the `CHANGELOG.md` section, then run the Release workflow
  (`gh workflow run release.yml --ref main`). Details in `CONTRIBUTING.md`.
- [fork] Take upstream changes with `git fetch upstream` and `git merge upstream/main`, never a rebase.
- [fork] `CHANGELOG.md` keeps a section per fork release on top of the upstream sections.

## Known gaps

- Linked windows (`link-window`) are not supported. `list-panes -a` lists each pane under one session only, so a
  linked window gets a row under one session, and going to it lets tmux pick the session. Don't add code that relies
  on a window id belonging to one session without handling this first.
- A turn interrupted with Esc, or by denying a prompt, stays "running" or "waiting" until `idle_prompt` turns it
  idle about 60 seconds later.
- The sidebar is not added to a zoomed window; it is added on the next visit.

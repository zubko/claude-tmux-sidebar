# claude-tmux-sidebar (fork)

How the code works and how to check it: see `CONTRIBUTING.md`. Run `cargo xtask ci` before a commit; every Rust file
stays at 200 lines or less.

## Known gaps

- Linked windows (`link-window`) are not supported. `list-panes -a` lists each pane under one session only, so a
  linked window gets a row under one session, and going to it lets tmux pick the session. Don't add code that relies
  on a window id belonging to one session without handling this first.

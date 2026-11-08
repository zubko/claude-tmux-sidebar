# Backlog

- When the current tmux window has no AI session, the sidebar has no row to highlight. Highlight the window's tmux
  session heading instead.
- Read the agent status from OSC 7501 (Program Status Protocol) when tmux supports it. With it, a program tells the
  terminal its state (idle, working, blocked, done, error) and a short message. Claude Code already sends it. The
  sidebar could then read each pane's status from tmux formats, for any agent and not only through the Claude Code
  hooks.
  - Spec: https://www.superlogical.com/rex/docs/build/program-status
  - tmux discussion: https://github.com/tmux/tmux/issues/5713 (closed; the maintainer is not sold on the protocol yet)
  - tmux PR we wait for: https://github.com/tmux/tmux/pull/5830 (open). It adds the formats `pane_program_state`,
    `pane_program_name` and `pane_program_message`.

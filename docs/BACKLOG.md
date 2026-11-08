# Backlog

## Read the agent status from OSC 7501 when tmux supports it

OSC 7501 (Program Status Protocol) is a new terminal standard. A program uses it to tell the terminal its state: idle,
working, blocked (waiting for the user), done or error, and a short message. Claude Code already sends it.

When tmux supports it, the sidebar can read the status of each pane from tmux formats instead of the Claude Code hooks.
This works for any agent or tool that sends OSC 7501, not only Claude Code.

- Spec: https://www.superlogical.com/rex/docs/build/program-status
- tmux discussion: https://github.com/tmux/tmux/issues/5713 (closed; the maintainer is not sold on the protocol yet)
- tmux PR we wait for: https://github.com/tmux/tmux/pull/5830 (open). It adds the formats `pane_program_state`,
  `pane_program_name` and `pane_program_message`.

# Backlog

## Show the prompt cache state on a Claude row

The prompt cache expires some time after the last request. The next prompt in a session with a cold cache costs
more and is slower. Show this on the row, so the user knows it before they type.

Options to consider:

- A: A small mark when the cache is cold. Simple and quiet. Probably the best default.
- B: The time left until the cache goes cold, or the time since it went cold.

Open points:

- The TTL is not fixed: 5 minutes or 1 hour, by account and settings. It may need a `@cts_` option.
- The start time is the last API request, not the last hook event. The transcript tail likely has it.
- A running session keeps the cache warm, so the mark matters only for done, waiting and idle rows.

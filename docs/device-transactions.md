# Owned device transactions

`PocketTransactionComponent` sends one bounded packet through the existing
asynchronous PocketStation transport. It can belong to a 3D actor, 2D actor or UI
widget. It contains no application protocol, installation rules or rewards.

Set its port, clear the packet and write bytes with `set_request_byte`. Call
`begin(length, length_at)` with 1-140 bytes; use `-1` for a fixed-length reply or
the protocol's reply-length byte position. The returned handle identifies both
the component and the request. Poll that handle across frames, or enable
`automatic_poll`. Inspect `snapshot` before reading `reply_byte`.

Busy requests reject staging edits. Wrong-component and stale handles cannot
poll, cancel or read another request. Disable and end play cancel owned work and
release the bus. A canceled host bridge request remains occupied until its late
reply arrives; a new request cannot steal that reply. Read `last_rejection` when
begin returns an invalid handle. A completed request's snapshot reports its own
transport error independently.

The selected C++, Blueprint or Lua project owns packet validation, producer
identity, timeouts between protocol steps, retry decisions and application state.
Persist rewards successfully before sending an acknowledgement that authorizes
the producer to discard them.

# Named memory-card payloads

The existing `MemoryCardLibrary` staging buffer supports all 4096 payload bytes.
Its metadata buffer is separate: clear it, stage up to 18 filename bytes and
128 title bytes, then use `read_staged_name` or `write_staged_named`. Terminate
short strings with a zero byte. The final filename/title sentinel remains zero.
Out-of-range indices and byte values are rejected without changing the buffer.

The memory-card service copies metadata and payload before returning an accepted
request, so later staging edits cannot change an in-flight save. Its existing
request IDs, alternating copies, checksum and verification behavior remain
authoritative. Project code owns the schema and handles successful, failed,
interrupted or stale completions. Generic indexed-save operations remain
available alongside named files.

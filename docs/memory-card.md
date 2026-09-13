# Memory Card service

`epok::memory_card` exposes asynchronous save operations to Behaviours. The
runtime owns the service for the application's lifetime, including scene changes.
Requests copy payloads, filenames, titles and icons before returning. Completions
update service status rather than calling a Behaviour that might have unloaded.
Do not retain `data()` across another request; copy a successful result into the
project's state before starting the next operation.

The service uses PsyQo MemoryCardFileSystem with AdvancedPad, which arbitrates
the shared controller/card bus. Ports are zero-based: `0` is the first physical
card slot, `1` is the second. Controller polling maps to AdvancedPad indices
`Pad1a=0` and `Pad2a=4`. While a filesystem transaction owns the bus, controller
readings retain their last sampled state. A save screen can pause gameplay via
`epok::time.set_paused()` and continue polling card status in `frame_update()`.

| Request | Behavior |
| --- | --- |
| `probe(port=0)` | Check that a formatted card is available. |
| `read(baseName, port=0)` | Read and validate both save copies, selecting the newest valid record. |
| `write(baseName, title, data, byteCount, port=0, icon=nullptr)` | Copy at most 4096 bytes, write the inactive copy, then read it back and verify it. |
| `list(port=0)` | List up to 15 physical files, available through `files()` and `file_count()`. |

Requests return `true` when accepted. Rejected requests report
`status().last_rejection` and preserve the status of an operation already in
progress. `busy()` is true until completion. `status().state` then becomes
`CardState::Succeeded` or `CardState::Failed`; `status().error` contains the result.
`status().request` identifies the latest accepted request and `completed` records
which request finished. `card_error_message(error)` provides a readable message.
`data()` and `size()` expose a successfully read or verified payload.

```cpp
// Serialize your own versioned payload into bytes; avoid pointers or native
// struct padding in a persistent file format.
uint8_t payload[4] = {1, 0, 0, 42};
bool accepted = epok::memory_card.write(
    "BASLUS-99999SAVE", "Adventure Save", payload, sizeof(payload));

// Later, in frame_update (which also runs while simulation is paused):
const auto& status = epok::memory_card.status();
if (status.state == epok::CardState::Succeeded) {
    // Record status.completed so this completion is handled only once.
} else if (status.state == epok::CardState::Failed) {
    // Display epok::card_error_message(status.error).
}
```

Base names contain 1–18 characters. Use a project-specific Sony-style identifier
to avoid colliding with another game's saves. UTF-8 titles accept at most 128
bytes and are converted by PsyQo for the BIOS title field. Optional `CardIcon`
data supplies 1–3 16×16 4bpp frames and a 16-color BGR555 palette; the default is a
plain white icon. The project's payload remains opaque to the engine.

## Interrupted write recovery

Each logical save alternates between physical files `baseName-A` and
`baseName-B`. Each record includes a format version, sequence, logical byte length
and CRC32 covering both metadata and payload. A write always targets the copy
other than the newest valid one, then verifies the written record. A missing,
truncated or corrupt newer copy falls back to the previous valid copy on read.
The first successful write occupies one 8KiB block; a subsequent write needs a
second block. Together the pair occupies two blocks of the card's 15 available
file blocks. Listing shows these physical files individually.

This avoids overwriting the last good record when the SDK reuses blocks during
an overwrite. A card removal or interrupted inactive-copy write is reported as
an error, while the previous valid copy remains available. The service does not
format cards, delete saves or retry writes automatically. `NoCard`,
`NotFormatted`, `OutOfSpace`, transfer errors, invalid data and verification
failure are returned to the project for its chosen UI/retry policy.

## Verification

`python tests/runtime/verify_spatial.py` tests the service with an asynchronous
driver that retains borrowed pointers exactly as the SDK does. Coverage includes
caller buffer mutation after submission, newest-copy selection, corrupted
metadata, interrupted writes, verification failures, bounds, missing/unformatted
cards, empty payloads, listing and both physical controller ports. The demo's
MIPS build also compiles and links the real PsyQo Memory Card backend. These tests
do not write to a user's physical or emulator memory card.

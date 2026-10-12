# PSX renderer and streaming research tools

This is an unfinished engineering checkpoint, not a claim of renderer fidelity
or sustained frame-rate parity. The reusable engine implementation is under
`src/`, `runtime/`, and `native/`; these utilities support offline verification,
Ghidra audits, emulator profiling, and editor MCP sessions.

No reference executable, disc, extracted level assets, saved emulator state,
session credentials, or private research reports are included. Source-specific
importers and their dependent checks are supplied privately, preserving their
asset identifiers and reproduction contracts.

## Verification

From the repository root:

```sh
python3 -m unittest discover -s research/psx-reference/tools -p 'test_*.py'
```

Some tools require Pillow or PyYAML. Integration captures additionally require
a configured emulator, a privately supplied project, and matching build/input
hashes. `original_emulator_session.py` now takes explicit `--exe`, `--cue`, and
`--disc` arguments; its executable hash and observer addresses remain specific
to the audited reference version, not arbitrary PSX software.

## Portability and safety

The source helpers were exercised on Windows. Several retain Windows default
paths; the engine's macOS setup does not make those helpers portable by itself.
Resolve paths from explicit arguments or local configuration before use. Never
transfer MCP bearer tokens or generated adapter/session credentials between
machines. Use fresh, loopback-only sessions and terminate only owned processes.

Keep input hashes, observer boundaries, camera/list identity, emulated cycle
counts, CPU time, presentation intervals, and host wall time distinct. A display
origin change is not proof of GPU completion or hardware frame rate. An imported
visibility list is not a general-purpose visibility baker.

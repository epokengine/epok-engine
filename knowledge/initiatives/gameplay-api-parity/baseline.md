# Gameplay API parity baseline

Captured for baseline commit `e3b56518175ddd633ccc962042a86d30d7275234`
on `develop`, before the parity implementation. The executable acceptance entrypoint
is `tests/integration/verify_gameplay_api_parity.py`.

## Host correctness

- `cargo test --locked`: 665 passed. Two pre-existing audio/package cases failed
  because `psxavenc` was unavailable and the corresponding golden package hash
  could not be reproduced.
- `tests/runtime/verify_spatial.py gameplay_api`: passed after providing PyYAML
  through the isolated Python environment.
- Pinned Clang reflection fixture: passed; the real reflection request parsed
  schema 10 during implementation.
- API documentation generation was blocked on this host by the Python binding's
  missing libclang discovery. This is an environment gate, not recorded as a pass.

## Performance evidence boundary

The historical skeletal measurements described in `docs/performance.md` are
context only. No isolated baseline/candidate target pair for this initiative has
yet been captured, and no physical-console result is inferred. Final validation
must record SDK/compiler identities, build flags, fixture hashes and at least
three emulator runs per gated workload before claiming performance certification.

# Naming decision required by F2

Source revision: `a2207788282cf943d9d12cfeab63b4897af808b2`.

A temporary executable in the detached source worktree copied the existing `sample_filename` function verbatim from `crates/ms-audio/src/orchestrator.rs` and called the existing `ms_core::midi::parse_sample_filename` for all notes 0–127, velocities 1, 64, 127, and round-robin values 0, 1, 2.

`cargo run -p ms-export --example naming_probe --locked` passed its assertions:

- All 384 names with rr0 parsed to the original note and velocity.
- All 768 names with rr1 or rr2 returned None.
- For example, `C4_v127.wav` parses but `C4_v127_rr1.wav` does not.

The parser attempts to parse the entire suffix after `_v` as a u8. The rendered writer appends `_rrN` for nonzero round-robin values. Neither implementation has been changed.

F2 states: “If the round trip fails for the existing implementations, stop and record the mismatch; do not change either function’s output format in this package”. The requested decision is whether to proceed with unchanged implementations, explicit rejection tests, and a README explanation. The user accepted the implementation judgment: preserve, test, and document this source behavior.

## Golden baseline preparation

The source library’s 48 tests and 20 format-validation tests passed. `docs/golden-baseline.json` records 46 output files produced by `tests/support/fixture.rs` adapted only for the source crate import paths. All 11 registered exporters and five inherent-method exporters ran. Fixtures are synthesized sine PCM WAVs, never rendered from plugins. flate2 is 1.1.9; Ableton gzip is decompressed before hashing. A second source export run matched the complete path set and all 46 SHA-256 values. The extracted-crate golden test now passes against the source ledger, proving equality for this fixture.

# multisample-formats

MIT-licensed Rust instrument model and 16 exporters, extracted without history from `multisamples` revision `a2207788282cf943d9d12cfeab63b4897af808b2`. Rust 1.93 or later; edition 2024. Version 0.1.0 has an unstable API.

Use the release from Git:

```toml
[dependencies]
multisample-formats = { git = "https://github.com/mattsp1290/multisample-formats", tag = "v0.1.0" }
```

## API

Construct an `Instrument` with `Instrument::builder`, `Group`, and `Zone`, or read rendered sample directories with `builder::build_instrument_from_dir`. Register exporters explicitly: `ExporterRegistry::new()` starts empty. `export_single` validates the instrument, writes metadata under `<exports>/<format.dir_name()>/<instrument>`, and copies referenced samples using `samples_subdir`. `FormatExporter` retains its five methods: `format`, `name`, `export`, `samples_subdir`, `validate`. Calling an exporter directly writes metadata only; call `util::copy_samples` yourself when external WAVs are needed.

```rust
use multisample_formats::{ExportFormat, ExporterRegistry, SfzExporter};
let mut registry = ExporterRegistry::new();
registry.register(Box::new(SfzExporter));
assert!(registry.get(ExportFormat::Sfz).is_some());
```

The `serde` feature enables serialization of `ExportFormat` using the original 11 variant names. Serde remains an unconditional dependency for the instrument model and JSON exporters. The five inherent-method exporters have no `ExportFormat` variant and cannot be registered.

## Exporters and current output

This extraction preserves existing output, including incomplete native-format implementations. The extension methods on `ExportFormat` describe target formats; the table lists files actually written. Output equivalence does not establish acceptance by target products.

| Exporter | Target product / format | Actual output | `samples_subdir()` / inherent sample directory |
| --- | --- | --- | --- |
| `SfzExporter` | SFZ samplers | `.sfz` | `samples` |
| `DecentSamplerExporter` | Decent Sampler | `.dspreset` | `samples` |
| `BitwigExporter` | Bitwig Studio | `multisample.xml` (not a ZIP `.multisample`) | `samples` |
| `NkiExporter` | Native Instruments Kontakt | `.nicnt` JSON descriptor (not native `.nki`) | `Samples` |
| `Sf2Exporter` | SoundFont 2 | `.sf2`, minimal structure with empty audio data | `None` |
| `MpcKeygroupExporter` | Akai MPC | `.xpm` | `Samples` |
| `KorgExporter` | Korg multisample | `.korgmultisample.json` descriptor | `samples` |
| `AbletonExporter` | Ableton Live Sampler | `.adg` gzip XML (target extension helper is `.adv`) | `Samples` |
| `TonverkExporter` | Elektron Tonverk | `.elmulti` TOML (target extension helper is `.tonverk`) | `samples` |
| `OpXyExporter` | Teenage Engineering OP-XY | `.opxy.json` descriptor | `samples` |
| `WavBundleExporter` | Generic WAV bundle | `manifest.json` plus copied WAVs | `samples` |
| `Exs24Exporter` | Apple Logic EXS24 / Sampler | `.exs24.json` descriptor (not native `.exs`) | inherent method `export_exs24`; `Samples` |
| `NnxtExporter` | Reason NN-XT | `.sxt` XML | inherent method `export_nnxt`; `Samples` |
| `TalExporter` | TAL-Sampler | `.talsmpl` XML | inherent method `export_tal`; `samples` |
| `Tx16wxExporter` | TX16Wx | `.txprog` XML | inherent method `export_tx16wx`; `Samples` |
| `Ten10MusicExporter` | 1010music blackbox / bitbox | `preset.xml` | inherent method `export_1010`; `samples` |

Inherent methods create sample directories but do not copy WAVs. `Sf2Exporter` currently embeds no input audio and therefore produces no playable sample content. Native product interoperability has not been validated as part of this extraction.

Format and product names are trademarks of their respective owners. This crate is not affiliated with or endorsed by those owners.

## Sample filenames

`naming::render_sample_filename(note, velocity, round_robin)` writes rendered input names such as `C4_v127.wav` or `C4_v127_rr1.wav`. MIDI notes 0–127 map to `C-1` through `G9`. `naming::parse_sample_filename` accepts only names without a round-robin suffix: it returns `Some((60, 127))` for `C4_v127.wav` and `None` for `C4_v127_rr1.wav` or `_rr2`. The directory builder consequently ignores those suffixed files. This existing source behavior was explicitly accepted during extraction; neither naming convention was changed.

`util::sample_filename(preset_name, note_name, velocity, round_robin)` writes export-side names with a sanitized preset prefix, such as `Init Patch_C4_v127.wav` or `Bass_A2_v64_rr1.wav`. These are not input names for the rendered-sample parser. Both writers include `_rrN` only for nonzero round-robin values.

## Testing

```sh
cargo build --all-targets --locked
cargo test --locked
cargo test --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --check
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features --locked
```

The source library had 48 unit tests; this crate preserves them and adds four naming tests (52 total). The source's 20 format-validation tests are retained. A feature-specific test checks serialized enum variant names. `tests/golden_tests.rs` checks the complete set of 46 files generated by a fixed three-zone instrument through all 16 exporters, using SHA-256 values recorded at the source revision in `docs/golden-baseline.json`. Fixtures are sine WAVs synthesized by test code, with no commercial plugin audio or resources. flate2 is pinned to the source lockfile version 1.1.9; Ableton gzip content is decompressed before comparison. No exporter is excluded as not byte-comparable; the complete ledger matched a second source export run.

Imported exporter modules retain narrowly specified Clippy allowances for existing style patterns, preserving their implementations during extraction. See [publication audit](docs/publication-audit.md) for import and dependency evidence.

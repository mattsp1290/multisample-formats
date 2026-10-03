# Publication audit

Source revision: `a2207788282cf943d9d12cfeab63b4897af808b2`. Import-path drift check from that revision to source HEAD was empty. This is a fresh allowlist import without private repository history.

## F1 scaffold — L5

Scaffold commit `d0cc593137209d1f952c13b3059bfe9c06f03838` passed [Ubuntu CI](https://github.com/mattsp1290/multisample-formats/actions/runs/37158728486). The initial baseline commit records the source revision. Its scaffold-only publication passed local build/test/clippy/format checks, package-list inspection, path/secret scans, dependency tree inspection, and cargo license.

## L3 — Allowlist and proprietary content

`docs/import-manifest.json` enumerates the 24 explicitly copied source/test files and source SHA-256 values, plus the three narrowly adapted helper sources. No whole source directory or git history was imported. No commercial presets, binaries, extracted resources, or plugin-rendered audio are present. Tests synthesize their WAV data. The commercial-product scan below was reviewed: every hit is a format/API name, descriptor compatibility explanation, or synthetic format test, not a plugin scanner, disassembly detail, or crash narrative. False matches on “variant” are enum commentary.

`git log --all --name-only --format=` was inspected: no history path matches `serum2_uidesc|\.fxp$|\.nki$|\.vital$|\.vstpreset$|\.vst3/`. The allowlist and working-tree package list also contain none of these paths. Repeated after the import commit: all committed history paths remain free of those matches.

Commercial-product scan (`serum|omnisphere|kontakt|vital|chipsynth|aria|spire|addictive|manis`, case-insensitive, src and tests):

```text
src/format.rs:8:     Kontakt,
src/format.rs:25:             Self::Kontakt => "nki",
src/format.rs:42:             Self::Kontakt => "NKI (Kontakt)",
src/format.rs:59:             Self::Kontakt => "Kontakt",
src/format.rs:76:             Self::Kontakt,
src/lib.rs:1: //! Export formats: SFZ, DecentSampler, Bitwig, Kontakt, SF2, MPC, Korg, Elektron Tonverk, and more.
src/model.rs:33: /// A group of zones (e.g., one velocity layer, or one round-robin variant).
src/formats/ten10music.rs:15: // In practice, this would need its own variant. For now, this module provides
src/formats/nki.rs:13: /// Exports instruments for Native Instruments Kontakt.
src/formats/nki.rs:17: /// that can be imported via Kontakt's Creator Tools or third-party converters.
src/formats/nki.rs:22:         ExportFormat::Kontakt
src/formats/nki.rs:26:         "NKI (Kontakt)"
src/formats/nki.rs:103:         "format": "kontakt-descriptor",
src/formats/nki.rs:124:         let inst = Instrument::builder("Kontakt Test")
src/formats/nki.rs:151:         let descriptor = dir.path().join("Kontakt Test").join("Kontakt Test.nicnt");
src/formats/nki.rs:156:         assert_eq!(content["format"], "kontakt-descriptor");
src/formats/nki.rs:157:         assert_eq!(content["instrument"]["name"], "Kontakt Test");
tests/format_validation_tests.rs:614:     // Kontakt is not registered
tests/format_validation_tests.rs:617:         ExportFormat::Kontakt,
tests/format_serde_tests.rs:5: fn serialized_variant_names_match_source_contract() {
tests/format_serde_tests.rs:10:         "Kontakt",
```

## L5 — Paths, secrets, package, dependencies

The scan below covers all intended repository text, excluding generated build output, git internals, and the audit's own quoted scan results. There are no local home paths or personal email credentials. Source/application-name references in README and decision/manifest documents are provenance; exported “Multisamples” generator labels are retained output bytes. The fixture wording describes synthetic source evidence and naming decisions. No reference links this crate to private source at build time.

Path/app-coupling scan (`/Users/|punk1290|mattsp1290@|multisamples`):

```text
src/formats/korg.rs:46:     let mut multisamples = Vec::new();
src/formats/korg.rs:50:             multisamples.push(json!({
src/formats/korg.rs:69:         "numSamples": multisamples.len(),
src/formats/korg.rs:70:         "multisamples": multisamples,
README.md:3: MIT-licensed Rust instrument model and 16 exporters, extracted without history from `multisamples` revision `a2207788282cf943d9d12cfeab63b4897af808b2`. Rust 1.93 or later; edition 2024. Version 0.1.0 has an unstable API.
```

Secret-pattern scan (`api[_-]?key|token|secret|password|datadog`, case-insensitive):

```text
src/util.rs:310:         std::fs::write(outside.join("secret.wav"), b"secret").unwrap();
src/util.rs:315:         let inst = inst_with_path(PathBuf::from("link/secret.wav"));
```

All secret hits are literal synthetic byte payloads and filenames in the retained symlink-escape regression test; none is a credential value.

`cargo package --list --allow-dirty` succeeded. The list below contains only crate code, tests, supporting JSON/Markdown evidence, license, README, lockfile, and Cargo-generated metadata; CI/review/build/private-source files are excluded.

```text
.cargo_vcs_info.json
Cargo.lock
Cargo.toml
Cargo.toml.orig
LICENSE
README.md
docs/golden-baseline.json
docs/import-manifest.json
docs/naming-decision.md
docs/publication-audit.md
src/builder.rs
src/format.rs
src/formats/ableton.rs
src/formats/bitwig.rs
src/formats/decent_sampler.rs
src/formats/exs24.rs
src/formats/korg.rs
src/formats/mod.rs
src/formats/mpc_keygroup.rs
src/formats/nki.rs
src/formats/nnxt.rs
src/formats/op_xy.rs
src/formats/sf2.rs
src/formats/sfz.rs
src/formats/tal.rs
src/formats/ten10music.rs
src/formats/tonverk.rs
src/formats/tx16wx.rs
src/formats/wav_bundle.rs
src/lib.rs
src/model.rs
src/naming.rs
src/registry.rs
src/traits.rs
src/util.rs
tests/format_serde_tests.rs
tests/format_validation_tests.rs
tests/golden_tests.rs
tests/support/fixture.rs
```

`cargo tree` succeeded and lists no ms-*, lotel-*, tauri, or specta crate. There are no path or git dependencies. `cargo license --json` succeeded. Every dependency offers MIT; unicode-ident also requires the permissive Unicode-3.0 notice. r-efi's LGPL alternative is not selected: MIT is offered. No copyleft-only dependency is included.

| Dependency | Version | Declared license expression |
| --- | --- | --- |
| adler2 | 2.0.1 | 0BSD OR Apache-2.0 OR MIT |
| bitflags | 2.13.2 | Apache-2.0 OR MIT |
| block-buffer | 0.10.4 | Apache-2.0 OR MIT |
| cfg-if | 1.0.5 | Apache-2.0 OR MIT |
| cpufeatures | 0.2.17 | Apache-2.0 OR MIT |
| crc32fast | 1.5.2 | Apache-2.0 OR MIT |
| crypto-common | 0.1.7 | Apache-2.0 OR MIT |
| digest | 0.10.7 | Apache-2.0 OR MIT |
| errno | 0.3.14 | Apache-2.0 OR MIT |
| fastrand | 2.5.0 | Apache-2.0 OR MIT |
| flate2 | 1.1.9 | Apache-2.0 OR MIT |
| generic-array | 0.14.7 | MIT |
| getrandom | 0.4.3 | Apache-2.0 OR MIT |
| itoa | 1.0.18 | Apache-2.0 OR MIT |
| libc | 0.2.190 | Apache-2.0 OR MIT |
| linux-raw-sys | 0.12.1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT |
| memchr | 2.8.3 | MIT OR Unlicense |
| miniz_oxide | 0.8.9 | Apache-2.0 OR MIT OR Zlib |
| multisample-formats | 0.1.0 | MIT |
| once_cell | 1.21.4 | Apache-2.0 OR MIT |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| proc-macro2 | 1.0.107 | Apache-2.0 OR MIT |
| quote | 1.0.47 | Apache-2.0 OR MIT |
| r-efi | 6.0.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT |
| rustix | 1.1.5 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT |
| serde | 1.0.229 | Apache-2.0 OR MIT |
| serde_core | 1.0.229 | Apache-2.0 OR MIT |
| serde_derive | 1.0.229 | Apache-2.0 OR MIT |
| serde_json | 1.0.151 | Apache-2.0 OR MIT |
| sha2 | 0.10.9 | Apache-2.0 OR MIT |
| simd-adler32 | 0.3.10 | MIT |
| syn | 2.0.119 | Apache-2.0 OR MIT |
| syn | 3.0.6 | Apache-2.0 OR MIT |
| tempfile | 3.27.0 | Apache-2.0 OR MIT |
| thiserror | 2.0.21 | Apache-2.0 OR MIT |
| thiserror-impl | 2.0.21 | Apache-2.0 OR MIT |
| tracing | 0.1.44 | MIT |
| tracing-attributes | 0.1.31 | MIT |
| tracing-core | 0.1.36 | MIT |
| typenum | 1.20.1 | Apache-2.0 OR MIT |
| unicode-ident | 1.0.26 | (Apache-2.0 OR MIT) AND Unicode-3.0 |
| version_check | 0.9.5 | Apache-2.0 OR MIT |
| windows-link | 0.2.1 | Apache-2.0 OR MIT |
| windows-sys | 0.61.2 | Apache-2.0 OR MIT |
| zmij | 1.0.23 | MIT |

## Regression evidence and accepted naming behavior

The source library had 48 unit tests. All 48 are retained; three relevant source naming tests and one exhaustive filename test bring this crate to 52. The source's 20 format-validation tests are retained. A feature test preserves the 11 serialized enum variant names. Golden hashes cover all 46 outputs from all 16 exporters, including five inherent-method exporters. The source baseline was run twice with equal output sets and hashes. Ableton gzip content is decompressed and flate2 stays at 1.1.9. The extracted golden test passed against that ledger.

The user accepted preserving the existing naming behavior: rr0 inputs parse, rr1/rr2 are rejected. The writer output and parser's accepted format are unchanged. See `docs/naming-decision.md` and README. FormatExporter still has exactly five methods. ExportFormat has the original 11 variants and four methods, no specta derive, and feature-gated serde derives. Narrow Clippy style allowances on imported exporter modules preserve their source implementations; formatting and documentation edits do not affect output.

## Publication decision and release

Local L3 and L5 checks passed. The user explicitly approved this completed audit and the first imported-code push before publication. Approval was given on 2026-10-03 after reviewing the audit at commit a62f6f17dfd2445420b60fd53ddd5eb06808bc30. Imported code will be published through the standard-review push checkpoint. Both independent standard reviewers approved with zero Critical or Important findings. Three deduplicated documentation suggestions were applied. The all-features suite also passed on Rust 1.93. The first standard-review push checkpoint, thermonuclear review, final exact-revision Ubuntu CI check, and annotated v0.1.0 tag remain pending.

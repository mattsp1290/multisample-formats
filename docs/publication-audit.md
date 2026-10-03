# Publication audit

## F1 scaffold — L5

No imported code in the scaffold commit. Local path, application-name, and secret-pattern scans of all scaffold text returned no hits. `cargo package --list --allow-dirty` listed only Cargo metadata, LICENSE, README, and src/lib.rs (plus Cargo-generated metadata). `cargo tree -e normal` contains no ms-*, lotel-*, tauri, or specta dependencies. `cargo license --json` succeeded: all dependencies offer MIT; unicode-ident additionally requires Unicode-3.0, a permissive notice license. r-efi offers MIT as an alternative to LGPL. No copyleft-only dependency is included.

Build, test, clippy with warnings denied, and formatting checks passed locally. Ubuntu CI verification will be recorded in Beans on the exact scaffold revision.

## Imported code — L3 and L5

Pending; no imported code approved for publication yet.

use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use crate::format::ExportFormat;

use crate::model::{Instrument, LoopMode, Zone};
use crate::traits::{ExportError, FormatExporter};
use crate::util::sanitize_filename;

/// Exports instruments to Elektron Tonverk .elmulti format.
///
/// The .elmulti format uses TOML with nested `[[key-zones]]` → `[[velocity-layers]]`
/// → [[sample-slots]] structure. The Tonverk auto-interpolates between key zones
/// so no explicit key range boundaries are written.
pub struct TonverkExporter;

impl FormatExporter for TonverkExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::ElektronTonverk
    }

    fn name(&self) -> &str {
        "Elektron Tonverk"
    }

    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        std::fs::create_dir_all(&dir)?;

        let toml = generate_elmulti(instrument);
        let filename = format!("{}.elmulti", sanitize_filename(&instrument.name));
        std::fs::write(dir.join(filename), toml)?;

        Ok(())
    }
}

fn generate_elmulti(instrument: &Instrument) -> String {
    let mut out = String::new();

    // Header
    writeln!(out, "# ELEKTRON MULTI-SAMPLE MAPPING FORMAT").unwrap();
    writeln!(out, "version = 0").unwrap();
    writeln!(out, "name = {}", to_toml_string(&instrument.name)).unwrap();
    writeln!(out).unwrap();

    // Group zones by root_note -> velocity -> Vec<Zone> (round-robin slots)
    // BTreeMap ensures sorted output by root note
    let mut key_zones: BTreeMap<u8, BTreeMap<u8, Vec<&Zone>>> = BTreeMap::new();

    for group in &instrument.groups {
        for zone in &group.zones {
            key_zones
                .entry(zone.root_note)
                .or_default()
                .entry(zone.velocity_high) // use velocity_high as the layer key
                .or_default()
                .push(zone);
        }
    }

    // Write key zones
    for (&root_note, velocity_layers) in &key_zones {
        writeln!(out, "[[key-zones]]").unwrap();
        writeln!(out, "pitch = {root_note}").unwrap();
        writeln!(out, "key-center = {root_note}.0").unwrap();
        writeln!(out).unwrap();

        for (&velocity, slots) in velocity_layers {
            writeln!(out, "[[key-zones.velocity-layers]]").unwrap();
            // Linear MIDI-to-float mapping. Factory Tonverk instruments use specific
            // thresholds (0.247, 0.494, 0.996) but linear mapping is correct for
            // user-recorded instruments with arbitrary velocity layers.
            let vel_float = velocity as f64 / 127.0;
            writeln!(out, "velocity = {vel_float:.7}").unwrap();
            writeln!(out, "strategy = 'Forward'").unwrap();
            writeln!(out).unwrap();

            for zone in slots {
                writeln!(out, "[[key-zones.velocity-layers.sample-slots]]").unwrap();
                writeln!(
                    out,
                    "sample = {}",
                    to_toml_string(&zone.sample_path.to_string_lossy())
                )
                .unwrap();

                // Loop parameters
                if let Some(ref li) = zone.loop_info {
                    match li.mode {
                        LoopMode::None => {}
                        LoopMode::Forward => {
                            write_tonverk_loop_params(&mut out, li);
                        }
                        // Tonverk only supports Forward — downgrade Backward/PingPong
                        _ => {
                            tracing::warn!(
                                mode = ?li.mode,
                                sample = %zone.sample_path.display(),
                                "Tonverk only supports forward loops; downgrading loop mode"
                            );
                            write_tonverk_loop_params(&mut out, li);
                        }
                    }
                }

                // Sample trim (used by Auto Sampler mode and our markers)
                if zone.sample_start > 0 {
                    writeln!(out, "trim-start = {}", zone.sample_start).unwrap();
                }
                if let Some(end) = zone.sample_end {
                    writeln!(out, "trim-end = {end}").unwrap();
                }

                writeln!(out).unwrap();
            }
        }
    }

    out
}

/// Write Tonverk loop parameters (always as Forward mode).
fn write_tonverk_loop_params(out: &mut String, li: &crate::model::LoopInfo) {
    writeln!(out, "loop-mode = 'Forward'").unwrap();
    writeln!(out, "loop-start = {}", li.start).unwrap();
    writeln!(out, "loop-end = {}", li.end).unwrap();
    if li.crossfade > 0 {
        writeln!(out, "loop-crossfade = {}", li.crossfade).unwrap();
    }
    if li.keep_looping_on_release {
        writeln!(out, "keep-looping-on-release = true").unwrap();
    }
}

/// Format a value as a TOML basic string (double-quoted) with proper escaping.
fn to_toml_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Group, LoopInfo, LoopMode, Zone};
    use std::path::PathBuf;

    fn test_instrument() -> Instrument {
        Instrument::builder("Tonverk Test")
            .group(Group {
                name: "Main".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![Zone {
                    sample_path: PathBuf::from("C4.wav"),
                    root_note: 60,
                    key_low: 54,
                    key_high: 66,
                    velocity_low: 0,
                    velocity_high: 127,
                    gain_db: 0.0,
                    tune_cents: 0.0,
                    loop_info: None,
                    sample_start: 0,
                    sample_end: None,
                    round_robin: 0,
                }],
            })
            .build()
    }

    #[test]
    fn export_creates_elmulti() {
        let dir = tempfile::tempdir().unwrap();
        let exporter = TonverkExporter;
        exporter.export(&test_instrument(), dir.path()).unwrap();

        let elmulti = dir.path().join("Tonverk Test").join("Tonverk Test.elmulti");
        assert!(elmulti.exists());

        let content = std::fs::read_to_string(&elmulti).unwrap();
        assert!(content.contains("version = 0"));
        assert!(content.contains("name = \"Tonverk Test\""));
    }

    #[test]
    fn elmulti_structure() {
        let toml = generate_elmulti(&test_instrument());
        assert!(toml.contains("[[key-zones]]"));
        assert!(toml.contains("pitch = 60"));
        assert!(toml.contains("key-center = 60.0"));
        assert!(toml.contains("[[key-zones.velocity-layers]]"));
        assert!(toml.contains("strategy = 'Forward'"));
        assert!(toml.contains("[[key-zones.velocity-layers.sample-slots]]"));
        assert!(toml.contains("sample = \"C4.wav\""));
    }

    #[test]
    fn elmulti_with_loop() {
        let inst = Instrument::builder("Loop Test")
            .group(Group {
                name: "Main".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![Zone {
                    sample_path: PathBuf::from("C4.wav"),
                    root_note: 60,
                    key_low: 54,
                    key_high: 66,
                    velocity_low: 0,
                    velocity_high: 127,
                    gain_db: 0.0,
                    tune_cents: 0.0,
                    loop_info: Some(LoopInfo {
                        mode: LoopMode::Forward,
                        start: 1000,
                        end: 5000,
                        crossfade: 200,
                        keep_looping_on_release: true,
                    }),
                    sample_start: 100,
                    sample_end: Some(6000),
                    round_robin: 0,
                }],
            })
            .build();

        let toml = generate_elmulti(&inst);
        assert!(toml.contains("loop-mode = 'Forward'"));
        assert!(toml.contains("loop-start = 1000"));
        assert!(toml.contains("loop-end = 5000"));
        assert!(toml.contains("loop-crossfade = 200"));
        assert!(toml.contains("keep-looping-on-release = true"));
        assert!(toml.contains("trim-start = 100"));
        assert!(toml.contains("trim-end = 6000"));
    }

    #[test]
    fn elmulti_with_sample_trim() {
        let inst = Instrument::builder("Trim Test")
            .group(Group {
                name: "Main".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![Zone {
                    sample_path: PathBuf::from("C4.wav"),
                    root_note: 60,
                    key_low: 54,
                    key_high: 66,
                    velocity_low: 0,
                    velocity_high: 127,
                    gain_db: 0.0,
                    tune_cents: 0.0,
                    loop_info: None,
                    sample_start: 500,
                    sample_end: Some(10000),
                    round_robin: 0,
                }],
            })
            .build();

        let toml = generate_elmulti(&inst);
        assert!(toml.contains("trim-start = 500"));
        assert!(toml.contains("trim-end = 10000"));
        // No loop params when loop_info is None
        assert!(!toml.contains("loop-mode"));
    }

    #[test]
    fn elmulti_multiple_zones() {
        let inst = Instrument::builder("Multi Zone")
            .group(Group {
                name: "Main".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![
                    Zone {
                        sample_path: PathBuf::from("C2.wav"),
                        root_note: 36,
                        key_low: 0,
                        key_high: 47,
                        velocity_low: 0,
                        velocity_high: 127,
                        gain_db: 0.0,
                        tune_cents: 0.0,
                        loop_info: None,
                        sample_start: 0,
                        sample_end: None,
                        round_robin: 0,
                    },
                    Zone {
                        sample_path: PathBuf::from("C4.wav"),
                        root_note: 60,
                        key_low: 48,
                        key_high: 71,
                        velocity_low: 0,
                        velocity_high: 127,
                        gain_db: 0.0,
                        tune_cents: 0.0,
                        loop_info: None,
                        sample_start: 0,
                        sample_end: None,
                        round_robin: 0,
                    },
                    Zone {
                        sample_path: PathBuf::from("C6.wav"),
                        root_note: 84,
                        key_low: 72,
                        key_high: 127,
                        velocity_low: 0,
                        velocity_high: 127,
                        gain_db: 0.0,
                        tune_cents: 0.0,
                        loop_info: None,
                        sample_start: 0,
                        sample_end: None,
                        round_robin: 0,
                    },
                ],
            })
            .build();

        let toml = generate_elmulti(&inst);
        // Should have 3 key-zones
        assert_eq!(toml.matches("[[key-zones]]").count(), 3);
        assert!(toml.contains("pitch = 36"));
        assert!(toml.contains("pitch = 60"));
        assert!(toml.contains("pitch = 84"));
    }
}

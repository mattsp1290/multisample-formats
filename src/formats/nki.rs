// Preserve source exporter implementation during extraction.
#![allow(clippy::collapsible_if)]

use std::path::Path;

use crate::format::ExportFormat;
use serde_json::json;

use crate::model::Instrument;
use crate::traits::{ExportError, FormatExporter};
use crate::util::sanitize_filename;

/// Exports instruments for Native Instruments Kontakt.
///
/// Since the NKI binary format is proprietary and undocumented,
/// this exporter produces a JSON-based descriptor alongside WAV samples
/// that can be imported via Kontakt's Creator Tools or third-party converters.
pub struct NkiExporter;

impl FormatExporter for NkiExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::Kontakt
    }

    fn name(&self) -> &str {
        "NKI (Kontakt)"
    }

    fn samples_subdir(&self) -> Option<&'static str> {
        Some("Samples")
    }

    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        std::fs::create_dir_all(&dir)?;

        let descriptor = generate_nki_descriptor(instrument);
        let filename = format!("{}.nicnt", sanitize_filename(&instrument.name));
        std::fs::write(
            dir.join(filename),
            serde_json::to_string_pretty(&descriptor).unwrap(),
        )?;

        Ok(())
    }
}

fn generate_nki_descriptor(instrument: &Instrument) -> serde_json::Value {
    let mut groups = Vec::new();

    for group in &instrument.groups {
        let mut zones = Vec::new();
        for zone in &group.zones {
            let mut z = json!({
                "file": format!("Samples/{}", zone.sample_path.display()),
                "rootKey": zone.root_note,
                "lowKey": zone.key_low,
                "highKey": zone.key_high,
                "lowVelocity": zone.velocity_low,
                "highVelocity": zone.velocity_high,
                "volume": zone.gain_db,
                "tune": zone.tune_cents,
            });

            if let Some(ref li) = zone.loop_info {
                if li.mode != crate::model::LoopMode::None {
                    z["loop"] = json!({
                        "start": li.start,
                        "end": li.end,
                        "crossfade": li.crossfade,
                        "keepLoopingOnRelease": li.keep_looping_on_release,
                        "mode": match li.mode {
                            crate::model::LoopMode::Forward => "forward",
                            crate::model::LoopMode::Backward => "backward",
                            crate::model::LoopMode::PingPong => "ping_pong",
                            crate::model::LoopMode::None => "none",
                        },
                    });
                }
            }

            if zone.sample_start > 0 {
                z["sampleStart"] = json!(zone.sample_start);
            }
            if let Some(end) = zone.sample_end {
                z["sampleEnd"] = json!(end);
            }

            if zone.round_robin > 0 {
                z["roundRobin"] = json!(zone.round_robin);
            }

            zones.push(z);
        }

        groups.push(json!({
            "name": group.name,
            "zones": zones,
        }));
    }

    json!({
        "format": "kontakt-descriptor",
        "version": 1,
        "instrument": {
            "name": instrument.name,
            "vendor": instrument.vendor,
            "category": instrument.category,
            "description": instrument.description,
        },
        "groups": groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Group, Zone};
    use std::path::PathBuf;

    #[test]
    fn export_creates_descriptor() {
        let dir = tempfile::tempdir().unwrap();
        let inst = Instrument::builder("Kontakt Test")
            .vendor("Test")
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
            .build();

        NkiExporter.export(&inst, dir.path()).unwrap();

        let descriptor = dir.path().join("Kontakt Test").join("Kontakt Test.nicnt");
        assert!(descriptor.exists());

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&descriptor).unwrap()).unwrap();
        assert_eq!(content["format"], "kontakt-descriptor");
        assert_eq!(content["instrument"]["name"], "Kontakt Test");
    }
}

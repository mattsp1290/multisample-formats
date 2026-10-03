use std::path::Path;

use crate::format::ExportFormat;
use serde_json::json;

use crate::model::Instrument;
use crate::traits::{ExportError, FormatExporter};
use crate::util::sanitize_filename;

/// Exports instruments as a directory of WAV files with a JSON manifest.
pub struct WavBundleExporter;

impl FormatExporter for WavBundleExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::WavBundle
    }

    fn name(&self) -> &str {
        "WAV Bundle"
    }

    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        std::fs::create_dir_all(&dir)?;

        // Generate manifest
        let manifest = generate_manifest(instrument);
        let manifest_path = dir.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )?;

        Ok(())
    }
}

fn generate_manifest(instrument: &Instrument) -> serde_json::Value {
    let mut zones = Vec::new();

    for group in &instrument.groups {
        for zone in &group.zones {
            zones.push(json!({
                "sample": zone.sample_path.to_string_lossy(),
                "root_note": zone.root_note,
                "key_low": zone.key_low,
                "key_high": zone.key_high,
                "velocity_low": zone.velocity_low,
                "velocity_high": zone.velocity_high,
                "gain_db": zone.gain_db,
                "tune_cents": zone.tune_cents,
                "round_robin": zone.round_robin,
                "group": group.name,
            }));
        }
    }

    json!({
        "name": instrument.name,
        "vendor": instrument.vendor,
        "description": instrument.description,
        "category": instrument.category,
        "total_zones": instrument.total_zones(),
        "zones": zones,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Group, Zone};
    use std::path::PathBuf;

    #[test]
    fn export_creates_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let inst = Instrument::builder("My Synth")
            .group(Group {
                name: "Main".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![Zone {
                    sample_path: PathBuf::from("C4_v127.wav"),
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

        let exporter = WavBundleExporter;
        exporter.export(&inst, dir.path()).unwrap();

        let manifest = dir.path().join("My Synth").join("manifest.json");
        assert!(manifest.exists());

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&manifest).unwrap()).unwrap();
        assert_eq!(content["name"], "My Synth");
        assert_eq!(content["total_zones"], 1);
    }
}

use std::path::Path;

use crate::format::ExportFormat;
use serde_json::json;

use crate::model::Instrument;
use crate::traits::{ExportError, FormatExporter};
use crate::util::sanitize_filename;

/// Exports instruments to Korg multisample format.
///
/// Generates a JSON descriptor with zone parameters compatible with
/// Korg's multisample specification. The actual .KMP binary format
/// is proprietary; this produces a structured interchange format
/// that can be converted to KMP using Korg's tools.
pub struct KorgExporter;

impl FormatExporter for KorgExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::KorgMultisample
    }

    fn name(&self) -> &str {
        "Korg Multisample"
    }

    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        std::fs::create_dir_all(&dir)?;

        let descriptor = generate_korg_descriptor(instrument);
        let filename = format!(
            "{}.korgmultisample.json",
            sanitize_filename(&instrument.name)
        );
        std::fs::write(
            dir.join(filename),
            serde_json::to_string_pretty(&descriptor).unwrap(),
        )?;

        Ok(())
    }
}

fn generate_korg_descriptor(instrument: &Instrument) -> serde_json::Value {
    let mut multisamples = Vec::new();

    for group in &instrument.groups {
        for zone in &group.zones {
            multisamples.push(json!({
                "sampleFileName": zone.sample_path.to_string_lossy(),
                "originalKey": zone.root_note,
                "topKey": zone.key_high,
                "bottomKey": zone.key_low,
                "topVelocity": zone.velocity_high,
                "bottomVelocity": zone.velocity_low,
                "tune": zone.tune_cents,
                "level": zone.gain_db,
            }));
        }
    }

    json!({
        "format": "korg-multisample",
        "version": 1,
        "name": instrument.name,
        "vendor": instrument.vendor,
        "category": instrument.category,
        "numSamples": multisamples.len(),
        "multisamples": multisamples,
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
        let inst = Instrument::builder("Korg Test")
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

        KorgExporter.export(&inst, dir.path()).unwrap();

        let desc = dir
            .path()
            .join("Korg Test")
            .join("Korg Test.korgmultisample.json");
        assert!(desc.exists());

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&desc).unwrap()).unwrap();
        assert_eq!(content["format"], "korg-multisample");
        assert_eq!(content["numSamples"], 1);
    }
}

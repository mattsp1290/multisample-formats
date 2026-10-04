// Preserve source exporter implementation during extraction.
#![allow(clippy::collapsible_if)]

use std::path::Path;

use serde_json::json;

use crate::model::Instrument;
use crate::traits::ExportError;
use crate::util::sanitize_filename;

/// Exports instruments for Apple Logic EXS24 / Sampler.
///
/// Since the EXS24 binary format is proprietary and undocumented,
/// this exporter produces a JSON-based descriptor alongside a samples
/// directory. The descriptor can be consumed by third-party converters
/// to produce native `.exs` instrument files.
pub struct Exs24Exporter;

impl Exs24Exporter {
    pub fn export_exs24(
        &self,
        instrument: &Instrument,
        output_dir: &Path,
    ) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        let samples_dir = dir.join("Samples");
        std::fs::create_dir_all(&samples_dir)?;

        let descriptor = generate_exs24_descriptor(instrument);
        let filename = format!("{}.exs24.json", sanitize_filename(&instrument.name));
        std::fs::write(
            dir.join(filename),
            serde_json::to_string_pretty(&descriptor).unwrap(),
        )?;

        Ok(())
    }
}

fn generate_exs24_descriptor(instrument: &Instrument) -> serde_json::Value {
    let mut zones = Vec::new();
    let mut zone_id = 0;

    for group in &instrument.groups {
        for zone in &group.zones {
            let mut z = json!({
                "id": zone_id,
                "sampleFile": format!("Samples/{}", zone.sample_path.display()),
                "keyRangeMin": zone.key_low,
                "keyRangeMax": zone.key_high,
                "rootKey": zone.root_note,
                "velocityRangeMin": zone.velocity_low,
                "velocityRangeMax": zone.velocity_high,
                "coarseTune": (zone.tune_cents / 100.0) as i32,
                "fineTune": (zone.tune_cents % 100.0) as i32,
                "volume": zone.gain_db,
                "groupName": group.name,
            });

            if let Some(ref li) = zone.loop_info {
                if li.mode != crate::model::LoopMode::None {
                    z["loop"] = json!({
                        "start": li.start,
                        "end": li.end,
                        "crossfade": li.crossfade,
                        "enabled": true,
                    });
                }
            }

            if zone.round_robin > 0 {
                z["roundRobin"] = json!(zone.round_robin);
            }

            zones.push(z);
            zone_id += 1;
        }
    }

    json!({
        "format": "exs24-descriptor",
        "version": 1,
        "instrument": {
            "name": instrument.name,
            "vendor": instrument.vendor,
            "category": instrument.category,
            "description": instrument.description,
        },
        "zoneCount": zone_id,
        "zones": zones,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Group, Zone};
    use std::path::PathBuf;

    #[test]
    fn export_creates_exs24_descriptor() {
        let dir = tempfile::tempdir().unwrap();
        let inst = Instrument::builder("EXS24 Test")
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

        let exporter = Exs24Exporter;
        exporter.export_exs24(&inst, dir.path()).unwrap();

        let descriptor = dir.path().join("EXS24 Test").join("EXS24 Test.exs24.json");
        assert!(descriptor.exists());

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&descriptor).unwrap()).unwrap();
        assert_eq!(content["format"], "exs24-descriptor");
        assert_eq!(content["instrument"]["name"], "EXS24 Test");
        assert_eq!(content["zoneCount"], 1);
    }
}

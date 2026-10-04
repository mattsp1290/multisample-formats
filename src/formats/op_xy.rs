// Preserve source exporter implementation during extraction.
#![allow(clippy::unnecessary_cast)]

use std::path::Path;

use crate::format::ExportFormat;
use serde_json::json;

use crate::model::Instrument;
use crate::traits::{ExportError, FormatExporter};
use crate::util::sanitize_filename;

/// Exports instruments to Teenage Engineering OP-XY multisample format.
///
/// The OP-XY format is a JSON descriptor alongside WAV sample files.
/// Samples should be 16-bit 44.1 kHz mono or stereo WAV.
pub struct OpXyExporter;

impl FormatExporter for OpXyExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::OpXy
    }

    fn name(&self) -> &str {
        "Teenage Engineering OP-XY"
    }

    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        std::fs::create_dir_all(&dir)?;

        let descriptor = generate_op_xy_descriptor(instrument);
        let filename = format!("{}.opxy.json", sanitize_filename(&instrument.name));
        std::fs::write(
            dir.join(filename),
            serde_json::to_string_pretty(&descriptor).unwrap(),
        )?;

        Ok(())
    }
}

fn generate_op_xy_descriptor(instrument: &Instrument) -> serde_json::Value {
    let mut samples = Vec::new();

    for group in &instrument.groups {
        for zone in &group.zones {
            let (has_loop, loop_start, loop_end) = match &zone.loop_info {
                Some(info) => (true, info.start as u64, info.end as u64),
                None => (false, 0, 0),
            };

            // Volume: convert dB gain to linear (0.0 to 1.0 range)
            let volume = 10.0_f64.powf(zone.gain_db / 20.0).clamp(0.0, 1.0);

            samples.push(json!({
                "file": zone.sample_path.to_string_lossy(),
                "note": zone.root_note,
                "lo": zone.key_low,
                "hi": zone.key_high,
                "vel_lo": zone.velocity_low,
                "vel_hi": zone.velocity_high,
                "tune": zone.tune_cents,
                "volume": (volume * 1000.0).round() / 1000.0,
                "loop": has_loop,
                "loop_start": loop_start,
                "loop_end": loop_end,
            }));
        }
    }

    json!({
        "type": "multisample",
        "version": 1,
        "name": instrument.name,
        "vendor": instrument.vendor,
        "category": instrument.category,
        "octave": 0,
        "samples": samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Group, LoopInfo, LoopMode, Zone};
    use std::path::PathBuf;

    fn test_instrument() -> Instrument {
        Instrument::builder("OP-XY Test Pad")
            .vendor("TestVendor")
            .category("Pad")
            .group(Group {
                name: "Main".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![
                    Zone {
                        sample_path: PathBuf::from("C3.wav"),
                        root_note: 48,
                        key_low: 36,
                        key_high: 54,
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
                        key_low: 55,
                        key_high: 66,
                        velocity_low: 0,
                        velocity_high: 127,
                        gain_db: -3.0,
                        tune_cents: 5.0,
                        loop_info: Some(LoopInfo {
                            start: 1000,
                            end: 50000,
                            crossfade: 100,
                            mode: LoopMode::Forward,
                            keep_looping_on_release: false,
                        }),
                        sample_start: 0,
                        sample_end: None,
                        round_robin: 0,
                    },
                ],
            })
            .build()
    }

    #[test]
    fn generate_descriptor_structure() {
        let descriptor = generate_op_xy_descriptor(&test_instrument());

        assert_eq!(descriptor["type"], "multisample");
        assert_eq!(descriptor["version"], 1);
        assert_eq!(descriptor["name"], "OP-XY Test Pad");
        assert_eq!(descriptor["vendor"], "TestVendor");
        assert_eq!(descriptor["category"], "Pad");
        assert_eq!(descriptor["octave"], 0);

        let samples = descriptor["samples"].as_array().unwrap();
        assert_eq!(samples.len(), 2);

        // First sample: no loop
        assert_eq!(samples[0]["file"], "C3.wav");
        assert_eq!(samples[0]["note"], 48);
        assert_eq!(samples[0]["lo"], 36);
        assert_eq!(samples[0]["hi"], 54);
        assert_eq!(samples[0]["loop"], false);
        assert_eq!(samples[0]["volume"], 1.0);

        // Second sample: with loop and gain
        assert_eq!(samples[1]["file"], "C4.wav");
        assert_eq!(samples[1]["note"], 60);
        assert_eq!(samples[1]["loop"], true);
        assert_eq!(samples[1]["loop_start"], 1000);
        assert_eq!(samples[1]["loop_end"], 50000);
        assert_eq!(samples[1]["tune"], 5.0);
        // -3dB ≈ 0.708
        let vol = samples[1]["volume"].as_f64().unwrap();
        assert!((vol - 0.708).abs() < 0.01);
    }

    #[test]
    fn export_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let exporter = OpXyExporter;
        exporter.export(&test_instrument(), dir.path()).unwrap();

        let descriptor = dir
            .path()
            .join("OP-XY Test Pad")
            .join("OP-XY Test Pad.opxy.json");
        assert!(descriptor.exists());

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&descriptor).unwrap()).unwrap();
        assert_eq!(content["type"], "multisample");
        assert_eq!(content["samples"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn volume_conversion() {
        // 0 dB → 1.0
        assert!((10.0_f64.powf(0.0 / 20.0) - 1.0).abs() < 1e-10);
        // -6 dB → ~0.501
        assert!((10.0_f64.powf(-6.0 / 20.0) - 0.501).abs() < 0.01);
        // -20 dB → 0.1
        assert!((10.0_f64.powf(-20.0 / 20.0) - 0.1).abs() < 1e-10);
    }
}

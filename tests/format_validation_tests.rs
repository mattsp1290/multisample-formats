//! Export format validation tests.
//!
//! Tests each exporter with known input and validates the output:
//! - Schema validation for XML-based formats (SFZ, DecentSampler, Bitwig)
//! - JSON validation for JSON-based formats (Tonverk, WavBundle)
//! - Edge cases: single note, full range, empty instrument

use std::path::PathBuf;

use multisample_formats::ExportFormat;
use multisample_formats::FormatExporter;
use multisample_formats::model::{Group, Instrument, LoopInfo, LoopMode, Zone};
use multisample_formats::registry::ExporterRegistry;
use multisample_formats::traits::ExportError;
use multisample_formats::{
    BitwigExporter, DecentSamplerExporter, SfzExporter, TonverkExporter, WavBundleExporter,
};

// --- Test instrument builders ---

fn single_note_instrument() -> Instrument {
    Instrument::builder("Single Note")
        .vendor("Test")
        .category("Test")
        .group(Group {
            name: "Main".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 0,
            velocity_high: 127,
            zones: vec![Zone {
                sample_path: PathBuf::from("C4_v127.wav"),
                root_note: 60,
                key_low: 0,
                key_high: 127,
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

fn full_range_instrument() -> Instrument {
    let notes = [24, 36, 48, 60, 72, 84, 96];
    let ranges = multisample_formats::model::compute_key_ranges(&notes);

    let zones: Vec<Zone> = ranges
        .iter()
        .map(|&(low, root, high)| Zone {
            sample_path: PathBuf::from(format!("note_{root}_v127.wav")),
            root_note: root,
            key_low: low,
            key_high: high,
            velocity_low: 0,
            velocity_high: 127,
            gain_db: 0.0,
            tune_cents: 0.0,
            loop_info: None,
            sample_start: 0,
            sample_end: None,
            round_robin: 0,
        })
        .collect();

    Instrument::builder("Full Range Piano")
        .vendor("Test Vendor")
        .description("A test piano with full range")
        .category("Piano")
        .group(Group {
            name: "Full Velocity".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 0,
            velocity_high: 127,
            zones,
        })
        .build()
}

fn multi_velocity_instrument() -> Instrument {
    Instrument::builder("Velocity Test")
        .vendor("Test")
        .group(Group {
            name: "Soft".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 0,
            velocity_high: 63,
            zones: vec![Zone {
                sample_path: PathBuf::from("C4_v32.wav"),
                root_note: 60,
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 63,
                gain_db: -3.0,
                tune_cents: 0.0,
                loop_info: None,
                sample_start: 0,
                sample_end: None,
                round_robin: 0,
            }],
        })
        .group(Group {
            name: "Loud".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 64,
            velocity_high: 127,
            zones: vec![Zone {
                sample_path: PathBuf::from("C4_v127.wav"),
                root_note: 60,
                key_low: 0,
                key_high: 127,
                velocity_low: 64,
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

fn looped_instrument() -> Instrument {
    Instrument::builder("Looped Pad")
        .vendor("Test")
        .group(Group {
            name: "Main".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 0,
            velocity_high: 127,
            zones: vec![Zone {
                sample_path: PathBuf::from("C4_v127.wav"),
                root_note: 60,
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                gain_db: 0.0,
                tune_cents: 5.0,
                loop_info: Some(LoopInfo {
                    mode: LoopMode::Forward,
                    start: 4410,
                    end: 44100,
                    crossfade: 1000,
                    keep_looping_on_release: false,
                }),
                sample_start: 0,
                sample_end: None,
                round_robin: 0,
            }],
        })
        .build()
}

fn round_robin_instrument() -> Instrument {
    Instrument::builder("RR Drums")
        .vendor("Test")
        .group(Group {
            name: "Main".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 0,
            velocity_high: 127,
            zones: vec![
                Zone {
                    sample_path: PathBuf::from("kick_rr1.wav"),
                    root_note: 36,
                    key_low: 36,
                    key_high: 36,
                    velocity_low: 0,
                    velocity_high: 127,
                    gain_db: 0.0,
                    tune_cents: 0.0,
                    loop_info: None,
                    sample_start: 0,
                    sample_end: None,
                    round_robin: 1,
                },
                Zone {
                    sample_path: PathBuf::from("kick_rr2.wav"),
                    root_note: 36,
                    key_low: 36,
                    key_high: 36,
                    velocity_low: 0,
                    velocity_high: 127,
                    gain_db: 0.0,
                    tune_cents: 0.0,
                    loop_info: None,
                    sample_start: 0,
                    sample_end: None,
                    round_robin: 2,
                },
            ],
        })
        .build()
}

fn empty_instrument() -> Instrument {
    Instrument::builder("Empty").build()
}

// --- Helper to create a registry with all exporters ---

fn full_registry() -> ExporterRegistry {
    let mut reg = ExporterRegistry::new();
    reg.register(Box::new(SfzExporter));
    reg.register(Box::new(DecentSamplerExporter));
    reg.register(Box::new(BitwigExporter));
    reg.register(Box::new(TonverkExporter));
    reg.register(Box::new(WavBundleExporter));
    reg
}

// =============================================
// SFZ FORMAT TESTS
// =============================================

#[test]
fn sfz_single_note() {
    let tmp = tempfile::tempdir().unwrap();
    SfzExporter
        .export(&single_note_instrument(), tmp.path())
        .unwrap();

    let sfz = std::fs::read_to_string(tmp.path().join("Single Note/Single Note.sfz")).unwrap();
    assert!(sfz.contains("<control>"));
    assert!(sfz.contains("default_path=samples/"));
    assert!(sfz.contains("<global>"));
    assert!(sfz.contains("<group>"));
    assert!(sfz.contains("<region>"));
    assert!(sfz.contains("sample=C4_v127.wav"));
    assert!(sfz.contains("pitch_keycenter=60"));
    assert!(sfz.contains("lokey=0"));
    assert!(sfz.contains("hikey=127"));
}

#[test]
fn sfz_full_range() {
    let tmp = tempfile::tempdir().unwrap();
    SfzExporter
        .export(&full_range_instrument(), tmp.path())
        .unwrap();

    let sfz =
        std::fs::read_to_string(tmp.path().join("Full Range Piano/Full Range Piano.sfz")).unwrap();

    // Should have 7 regions
    let region_count = sfz.matches("<region>").count();
    assert_eq!(region_count, 7);

    // Verify vendor in header
    assert!(sfz.contains("Vendor: Test Vendor"));
}

#[test]
fn sfz_velocity_layers() {
    let tmp = tempfile::tempdir().unwrap();
    SfzExporter
        .export(&multi_velocity_instrument(), tmp.path())
        .unwrap();

    let sfz = std::fs::read_to_string(tmp.path().join("Velocity Test/Velocity Test.sfz")).unwrap();

    // Should have velocity ranges in groups
    assert!(sfz.contains("lovel=0"));
    assert!(sfz.contains("hivel=63"));
    assert!(sfz.contains("lovel=64"));
    assert!(sfz.contains("hivel=127"));
    // Gain should appear
    assert!(sfz.contains("volume=-3.0"));
}

#[test]
fn sfz_loop_info() {
    let tmp = tempfile::tempdir().unwrap();
    SfzExporter
        .export(&looped_instrument(), tmp.path())
        .unwrap();

    let sfz = std::fs::read_to_string(tmp.path().join("Looped Pad/Looped Pad.sfz")).unwrap();

    assert!(sfz.contains("loop_mode=loop_continuous"));
    assert!(sfz.contains("loop_start=4410"));
    assert!(sfz.contains("loop_end=44100"));
    assert!(sfz.contains("loop_crossfade=1000"));
    assert!(sfz.contains("tune=5"));
}

#[test]
fn sfz_round_robin() {
    let tmp = tempfile::tempdir().unwrap();
    SfzExporter
        .export(&round_robin_instrument(), tmp.path())
        .unwrap();

    let sfz = std::fs::read_to_string(tmp.path().join("RR Drums/RR Drums.sfz")).unwrap();

    assert!(sfz.contains("seq_position=1"));
    assert!(sfz.contains("seq_position=2"));
}

// =============================================
// DECENTSAMPLER FORMAT TESTS
// =============================================

#[test]
fn dspreset_single_note() {
    let tmp = tempfile::tempdir().unwrap();
    DecentSamplerExporter
        .export(&single_note_instrument(), tmp.path())
        .unwrap();

    let xml = std::fs::read_to_string(tmp.path().join("Single Note/Single Note.dspreset")).unwrap();

    assert!(xml.contains("<?xml version="));
    assert!(xml.contains("<DecentSampler>"));
    assert!(xml.contains("</DecentSampler>"));
    assert!(xml.contains("<groups>"));
    assert!(xml.contains(r#"rootNote="60""#));
    assert!(xml.contains(r#"loNote="0""#));
    assert!(xml.contains(r#"hiNote="127""#));
    assert!(xml.contains(r#"path="samples/C4_v127.wav""#));
}

#[test]
fn dspreset_velocity_layers() {
    let tmp = tempfile::tempdir().unwrap();
    DecentSamplerExporter
        .export(&multi_velocity_instrument(), tmp.path())
        .unwrap();

    let xml =
        std::fs::read_to_string(tmp.path().join("Velocity Test/Velocity Test.dspreset")).unwrap();

    assert!(xml.contains(r#"loVel="0""#));
    assert!(xml.contains(r#"hiVel="63""#));
    assert!(xml.contains(r#"volume="-3.0dB""#));
}

#[test]
fn dspreset_loop_info() {
    let tmp = tempfile::tempdir().unwrap();
    DecentSamplerExporter
        .export(&looped_instrument(), tmp.path())
        .unwrap();

    let xml = std::fs::read_to_string(tmp.path().join("Looped Pad/Looped Pad.dspreset")).unwrap();

    assert!(xml.contains(r#"loopStart="4410""#));
    assert!(xml.contains(r#"loopEnd="44100""#));
    assert!(xml.contains(r#"loopEnabled="true""#));
    assert!(xml.contains(r#"loopCrossfade="1000""#));
    assert!(xml.contains(r#"tuning="5""#));
}

// =============================================
// BITWIG FORMAT TESTS
// =============================================

#[test]
fn bitwig_single_note() {
    let tmp = tempfile::tempdir().unwrap();
    BitwigExporter
        .export(&single_note_instrument(), tmp.path())
        .unwrap();

    let xml = std::fs::read_to_string(tmp.path().join("Single Note/multisample.xml")).unwrap();

    assert!(xml.contains("<?xml version="));
    assert!(xml.contains(r#"<multisample name="Single Note">"#));
    assert!(xml.contains("<generator>Multisamples</generator>"));
    assert!(xml.contains(r#"root="60""#));
    assert!(xml.contains(r#"low="0""#));
    assert!(xml.contains(r#"high="127""#));
    assert!(xml.contains("</multisample>"));
}

#[test]
fn bitwig_full_range() {
    let tmp = tempfile::tempdir().unwrap();
    BitwigExporter
        .export(&full_range_instrument(), tmp.path())
        .unwrap();

    let xml = std::fs::read_to_string(tmp.path().join("Full Range Piano/multisample.xml")).unwrap();

    // Should have 7 sample elements
    let sample_count = xml.matches("<sample ").count();
    assert_eq!(sample_count, 7);

    assert!(xml.contains(r#"<category>Piano</category>"#));
    assert!(xml.contains(r#"<creator>Test Vendor</creator>"#));
}

#[test]
fn bitwig_loop_info() {
    let tmp = tempfile::tempdir().unwrap();
    BitwigExporter
        .export(&looped_instrument(), tmp.path())
        .unwrap();

    let xml = std::fs::read_to_string(tmp.path().join("Looped Pad/multisample.xml")).unwrap();

    assert!(xml.contains(r#"<loop start="4410" end="44100" crossfade="1000" />"#));
}

// =============================================
// TONVERK FORMAT TESTS
// =============================================

#[test]
fn tonverk_single_note() {
    let tmp = tempfile::tempdir().unwrap();
    TonverkExporter
        .export(&single_note_instrument(), tmp.path())
        .unwrap();

    let content =
        std::fs::read_to_string(tmp.path().join("Single Note/Single Note.elmulti")).unwrap();

    assert!(content.contains("version = 0"));
    assert!(content.contains("name = \"Single Note\""));
    assert!(content.contains("[[key-zones]]"));
    assert!(content.contains("pitch = 60"));
    assert!(content.contains("sample = \"C4_v127.wav\""));
}

#[test]
fn tonverk_full_range() {
    let tmp = tempfile::tempdir().unwrap();
    TonverkExporter
        .export(&full_range_instrument(), tmp.path())
        .unwrap();

    let content =
        std::fs::read_to_string(tmp.path().join("Full Range Piano/Full Range Piano.elmulti"))
            .unwrap();

    assert!(content.contains("name = \"Full Range Piano\""));
    // 7 distinct root notes → 7 [[key-zones]] entries
    assert_eq!(content.matches("[[key-zones]]").count(), 7);
    assert!(content.contains("pitch = 24"));
    assert!(content.contains("pitch = 96"));
}

// =============================================
// WAV BUNDLE FORMAT TESTS
// =============================================

#[test]
fn wav_bundle_single_note() {
    let tmp = tempfile::tempdir().unwrap();
    WavBundleExporter
        .export(&single_note_instrument(), tmp.path())
        .unwrap();

    // Check manifest
    let content = std::fs::read_to_string(tmp.path().join("Single Note/manifest.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(json["name"], "Single Note");
    assert_eq!(json["total_zones"], 1);

    let zones = json["zones"].as_array().unwrap();
    assert_eq!(zones[0]["root_note"], 60);
    assert_eq!(zones[0]["group"], "Main");

    // samples dir is now created by the registry's copy_samples step, not by the exporter
}

#[test]
fn wav_bundle_round_robin() {
    let tmp = tempfile::tempdir().unwrap();
    WavBundleExporter
        .export(&round_robin_instrument(), tmp.path())
        .unwrap();

    let content = std::fs::read_to_string(tmp.path().join("RR Drums/manifest.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let zones = json["zones"].as_array().unwrap();
    assert_eq!(zones.len(), 2);
    assert_eq!(zones[0]["round_robin"], 1);
    assert_eq!(zones[1]["round_robin"], 2);
}

// =============================================
// EDGE CASE TESTS
// =============================================

#[test]
fn empty_instrument_rejected_by_all_exporters() {
    let reg = full_registry();
    let inst = empty_instrument();

    for fmt in [
        ExportFormat::Sfz,
        ExportFormat::DecentSampler,
        ExportFormat::BitwigMultisample,
        ExportFormat::ElektronTonverk,
        ExportFormat::WavBundle,
    ] {
        let result = reg.export_single(
            &inst,
            fmt,
            std::path::Path::new("/tmp/test_empty"),
            std::path::Path::new("/tmp/test_empty_exports"),
        );
        assert!(
            matches!(result, Err(ExportError::EmptyInstrument)),
            "format {} should reject empty instrument",
            fmt.display_name()
        );
    }
}

#[test]
fn multi_format_export() {
    let tmp = tempfile::tempdir().unwrap();
    let reg = full_registry();
    let inst = single_note_instrument();

    // Create fake WAV so copy_samples can find it
    std::fs::write(tmp.path().join("C4_v127.wav"), b"RIFF fake wav").unwrap();

    let formats = vec![
        ExportFormat::Sfz,
        ExportFormat::DecentSampler,
        ExportFormat::BitwigMultisample,
        ExportFormat::ElektronTonverk,
        ExportFormat::WavBundle,
    ];

    let exports = tmp.path().join("exports");
    let results = reg.export_multi(&inst, &formats, tmp.path(), &exports);
    for (fmt, result) in &results {
        assert!(
            result.is_ok(),
            "export failed for {}: {:?}",
            fmt.display_name(),
            result
        );
    }
}

#[test]
fn special_characters_in_name() {
    let inst = Instrument::builder("Test: A/B <Synth>")
        .vendor("Vendor")
        .group(Group {
            name: "Main".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 0,
            velocity_high: 127,
            zones: vec![Zone {
                sample_path: PathBuf::from("C4.wav"),
                root_note: 60,
                key_low: 0,
                key_high: 127,
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

    let tmp = tempfile::tempdir().unwrap();

    // All exporters should handle special characters
    SfzExporter.export(&inst, tmp.path()).unwrap();
    DecentSamplerExporter.export(&inst, tmp.path()).unwrap();
    BitwigExporter.export(&inst, tmp.path()).unwrap();
    TonverkExporter.export(&inst, tmp.path()).unwrap();
    WavBundleExporter.export(&inst, tmp.path()).unwrap();

    // Sanitized directory should exist
    let sanitized = tmp.path().join("Test_ A_B _Synth_");
    assert!(sanitized.is_dir());
}

// =============================================
// REGISTRY TESTS
// =============================================

#[test]
fn registry_lists_all_registered_formats() {
    let reg = full_registry();
    let formats = reg.formats();
    assert_eq!(formats.len(), 5);
}

#[test]
fn registry_unregistered_format_errors() {
    let reg = full_registry();
    let inst = single_note_instrument();
    // Kontakt is not registered
    let result = reg.export_single(
        &inst,
        ExportFormat::Kontakt,
        std::path::Path::new("/tmp"),
        std::path::Path::new("/tmp/exports"),
    );
    assert!(result.is_err());
}

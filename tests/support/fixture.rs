use multisample_formats::*;
use std::path::Path;

/// A fixed three-zone instrument with synthesized PCM sine waves only.
pub fn export_fixture(root: &Path) {
    let source = root.join("source");
    std::fs::create_dir_all(&source).unwrap();
    let zones = [(48, 0, 54), (60, 55, 66), (72, 67, 127)]
        .into_iter()
        .map(|(note, low, high)| {
            let filename = format!("note_{note}.wav");
            std::fs::write(source.join(&filename), sine_wav(note)).unwrap();
            Zone {
                sample_path: filename.into(),
                root_note: note,
                key_low: low,
                key_high: high,
                velocity_low: 1,
                velocity_high: 127,
                gain_db: -3.0,
                tune_cents: 2.0,
                loop_info: Some(LoopInfo {
                    mode: LoopMode::Forward,
                    start: 8,
                    end: 96,
                    crossfade: 4,
                    keep_looping_on_release: true,
                }),
                sample_start: 2,
                sample_end: Some(120),
                round_robin: 0,
            }
        })
        .collect();
    let instrument = Instrument::builder("Golden Sine")
        .vendor("Fixture")
        .category("Test")
        .description("Synthetic three-zone fixture")
        .group(Group {
            name: "Main".into(),
            key_low: 0,
            key_high: 127,
            velocity_low: 1,
            velocity_high: 127,
            zones,
        })
        .build();
    let mut registry = ExporterRegistry::new();
    for exporter in [
        Box::new(SfzExporter) as Box<dyn FormatExporter>,
        Box::new(DecentSamplerExporter),
        Box::new(BitwigExporter),
        Box::new(NkiExporter),
        Box::new(Sf2Exporter),
        Box::new(MpcKeygroupExporter),
        Box::new(KorgExporter),
        Box::new(AbletonExporter),
        Box::new(TonverkExporter),
        Box::new(OpXyExporter),
        Box::new(WavBundleExporter),
    ] {
        registry.register(exporter);
    }
    let exports = root.join("exports");
    for &format in ExportFormat::all() {
        registry
            .export_single(&instrument, format, &source, &exports)
            .unwrap();
    }
    Exs24Exporter
        .export_exs24(&instrument, &exports.join("EXS24"))
        .unwrap();
    NnxtExporter
        .export_nnxt(&instrument, &exports.join("NNXT"))
        .unwrap();
    TalExporter
        .export_tal(&instrument, &exports.join("TAL"))
        .unwrap();
    Tx16wxExporter
        .export_tx16wx(&instrument, &exports.join("TX16Wx"))
        .unwrap();
    Ten10MusicExporter
        .export_1010(&instrument, &exports.join("1010music"))
        .unwrap();
}

fn sine_wav(note: u8) -> Vec<u8> {
    let frames = 128u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + frames * 2).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&44100u32.to_le_bytes());
    bytes.extend_from_slice(&88200u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(frames * 2).to_le_bytes());
    for frame in 0..frames {
        let phase = std::f64::consts::TAU * f64::from(frame) * f64::from(note) / 128.0;
        let sample = (phase.sin() * 8191.0).round() as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

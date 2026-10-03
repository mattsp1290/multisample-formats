// Preserve source exporter implementation during extraction.
#![allow(clippy::collapsible_if)]

use std::path::Path;

use crate::format::ExportFormat;

use crate::model::Instrument;
use crate::traits::{ExportError, FormatExporter};
use crate::util::sanitize_filename;

/// Exports instruments to Bitwig .multisample format.
///
/// Bitwig multisample is a ZIP archive containing:
/// - multisample.xml (metadata and zone mappings)
/// - WAV sample files
pub struct BitwigExporter;

impl FormatExporter for BitwigExporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::BitwigMultisample
    }

    fn name(&self) -> &str {
        "Bitwig Multisample"
    }

    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        std::fs::create_dir_all(&dir)?;

        let xml = generate_multisample_xml(instrument);
        std::fs::write(dir.join("multisample.xml"), xml)?;

        // Note: A complete implementation would create a ZIP archive.
        // For now, we write the XML descriptor alongside the samples.
        // The ZIP packaging can be added when the zip crate is available.

        Ok(())
    }
}

fn generate_multisample_xml(instrument: &Instrument) -> String {
    use std::fmt::Write;

    let mut xml = String::new();
    writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();
    writeln!(xml, r#"<multisample name="{}">"#, instrument.name).unwrap();
    writeln!(xml, r#"  <generator>Multisamples</generator>"#).unwrap();
    writeln!(xml, r#"  <category>{}</category>"#, instrument.category).unwrap();
    writeln!(xml, r#"  <creator>{}</creator>"#, instrument.vendor).unwrap();
    writeln!(
        xml,
        r#"  <description>{}</description>"#,
        instrument.description
    )
    .unwrap();

    for group in &instrument.groups {
        for zone in &group.zones {
            writeln!(
                xml,
                r#"  <sample file="{}" gain="{:.2}" sample-start="0">"#,
                zone.sample_path.display(),
                zone.gain_db,
            )
            .unwrap();
            writeln!(
                xml,
                r#"    <key root="{}" low="{}" high="{}" />"#,
                zone.root_note, zone.key_low, zone.key_high
            )
            .unwrap();
            writeln!(
                xml,
                r#"    <velocity low="{}" high="{}" />"#,
                zone.velocity_low, zone.velocity_high
            )
            .unwrap();
            if let Some(ref li) = zone.loop_info {
                if li.mode != crate::model::LoopMode::None {
                    writeln!(
                        xml,
                        r#"    <loop start="{}" end="{}" crossfade="{}" />"#,
                        li.start, li.end, li.crossfade
                    )
                    .unwrap();
                }
            }
            writeln!(xml, r#"  </sample>"#).unwrap();
        }
    }

    writeln!(xml, r#"</multisample>"#).unwrap();
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Group, Zone};
    use std::path::PathBuf;

    #[test]
    fn generates_valid_xml() {
        let inst = Instrument::builder("My Synth")
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

        let xml = generate_multisample_xml(&inst);
        assert!(xml.contains(r#"name="My Synth""#));
        assert!(xml.contains(r#"root="60""#));
    }
}

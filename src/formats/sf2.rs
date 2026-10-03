// Preserve source exporter implementation during extraction.
#![allow(clippy::manual_is_multiple_of, clippy::too_many_arguments)]

use std::path::Path;

use crate::format::ExportFormat;

use crate::model::Instrument;
use crate::traits::{ExportError, FormatExporter};
use crate::util::sanitize_filename;

/// Exports instruments to SoundFont 2 (.sf2) format.
///
/// SF2 is a RIFF-based binary format containing:
/// - INFO chunk: metadata
/// - sdta chunk: sample data (raw PCM)
/// - pdta chunk: preset/instrument/zone headers
///
/// This implementation generates a minimal SF2 file structure
/// with the correct RIFF headers and zone mappings.
pub struct Sf2Exporter;

impl FormatExporter for Sf2Exporter {
    fn format(&self) -> ExportFormat {
        ExportFormat::SoundFont2
    }

    fn name(&self) -> &str {
        "SoundFont 2"
    }

    fn samples_subdir(&self) -> Option<&'static str> {
        None
    }

    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError> {
        let dir = output_dir.join(sanitize_filename(&instrument.name));
        std::fs::create_dir_all(&dir)?;

        let sf2_data = generate_sf2(instrument);
        let filename = format!("{}.sf2", sanitize_filename(&instrument.name));
        std::fs::write(dir.join(filename), sf2_data)?;

        Ok(())
    }
}

/// Generate a minimal SF2 binary file.
///
/// The SF2 spec (v2.04) defines the RIFF structure:
/// RIFF('sfbk'
///   LIST('INFO' ...)
///   LIST('sdta' smpl(...))
///   LIST('pdta' phdr ihdr shdr bag gen mod ...))
fn generate_sf2(instrument: &Instrument) -> Vec<u8> {
    let mut buf = Vec::new();

    // We build the three sub-chunks first, then wrap in RIFF
    let info_chunk = build_info_chunk(instrument);
    let sdta_chunk = build_sdta_chunk();
    let pdta_chunk = build_pdta_chunk(instrument);

    // RIFF header
    let total_size = 4 + info_chunk.len() + sdta_chunk.len() + pdta_chunk.len(); // 'sfbk' + chunks
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(total_size as u32).to_le_bytes());
    buf.extend_from_slice(b"sfbk");

    buf.extend_from_slice(&info_chunk);
    buf.extend_from_slice(&sdta_chunk);
    buf.extend_from_slice(&pdta_chunk);

    buf
}

fn build_info_chunk(instrument: &Instrument) -> Vec<u8> {
    let mut sub = Vec::new();

    // ifil: SF version (2.04)
    write_sub_chunk(&mut sub, b"ifil", &[4, 0, 2, 0]); // major=2, minor=4 (LE u16 pairs)

    // isng: sound engine
    write_sub_chunk_str(&mut sub, b"isng", "EMU8000");

    // INAM: bank name
    write_sub_chunk_str(&mut sub, b"INAM", &instrument.name);

    // ISFT: software
    write_sub_chunk_str(&mut sub, b"ISFT", "Multisamples");

    let mut chunk = Vec::new();
    chunk.extend_from_slice(b"LIST");
    chunk.extend_from_slice(&((4 + sub.len()) as u32).to_le_bytes());
    chunk.extend_from_slice(b"INFO");
    chunk.extend_from_slice(&sub);
    chunk
}

fn build_sdta_chunk() -> Vec<u8> {
    // Minimal sdta with empty sample data + 46 zero samples (terminal)
    let sample_data = vec![0u8; 92]; // 46 samples × 2 bytes (16-bit)

    let mut sub = Vec::new();
    write_sub_chunk(&mut sub, b"smpl", &sample_data);

    let mut chunk = Vec::new();
    chunk.extend_from_slice(b"LIST");
    chunk.extend_from_slice(&((4 + sub.len()) as u32).to_le_bytes());
    chunk.extend_from_slice(b"sdta");
    chunk.extend_from_slice(&sub);
    chunk
}

fn build_pdta_chunk(instrument: &Instrument) -> Vec<u8> {
    let mut sub = Vec::new();

    // phdr: preset headers (one preset + terminal EOP)
    let mut phdr = Vec::new();
    write_phdr_record(&mut phdr, &instrument.name, 0, 0, 0, 0, 0);
    write_phdr_record(&mut phdr, "EOP", 0, 0, 0, 0, 0); // terminal
    write_sub_chunk(&mut sub, b"phdr", &phdr);

    // pbag: preset bags
    let mut pbag = Vec::new();
    write_bag_record(&mut pbag, 0, 0);
    write_bag_record(&mut pbag, 0, 0); // terminal
    write_sub_chunk(&mut sub, b"pbag", &pbag);

    // pmod: preset modulators (empty + terminal)
    let pmod = vec![0u8; 10]; // one terminal record
    write_sub_chunk(&mut sub, b"pmod", &pmod);

    // pgen: preset generators (empty + terminal)
    let pgen = vec![0u8; 4]; // one terminal record
    write_sub_chunk(&mut sub, b"pgen", &pgen);

    // inst: instrument headers
    let mut inst = Vec::new();
    write_inst_record(&mut inst, &instrument.name, 0);
    write_inst_record(&mut inst, "EOI", 0); // terminal
    write_sub_chunk(&mut sub, b"inst", &inst);

    // ibag: instrument bags
    let zone_count = instrument.total_zones().max(1);
    let mut ibag = Vec::new();
    for i in 0..=zone_count {
        write_bag_record(&mut ibag, (i * 2) as u16, 0);
    }
    write_sub_chunk(&mut sub, b"ibag", &ibag);

    // imod: instrument modulators (empty + terminal)
    let imod = vec![0u8; 10];
    write_sub_chunk(&mut sub, b"imod", &imod);

    // igen: instrument generators (zone assignments)
    let mut igen = Vec::new();
    for group in &instrument.groups {
        for zone in &group.zones {
            // keyRange generator (op=43)
            igen.extend_from_slice(&43u16.to_le_bytes());
            igen.push(zone.key_low);
            igen.push(zone.key_high);
            // sampleID generator (op=53)
            igen.extend_from_slice(&53u16.to_le_bytes());
            igen.extend_from_slice(&0u16.to_le_bytes()); // sample index 0
        }
    }
    // Terminal
    igen.extend_from_slice(&0u16.to_le_bytes());
    igen.extend_from_slice(&0u16.to_le_bytes());
    write_sub_chunk(&mut sub, b"igen", &igen);

    // shdr: sample headers
    let mut shdr = Vec::new();
    write_shdr_record(&mut shdr, "sample", 0, 0, 0, 0, 44100, 60, 0);
    write_shdr_record(&mut shdr, "EOS", 0, 0, 0, 0, 0, 0, 0); // terminal
    write_sub_chunk(&mut sub, b"shdr", &shdr);

    let mut chunk = Vec::new();
    chunk.extend_from_slice(b"LIST");
    chunk.extend_from_slice(&((4 + sub.len()) as u32).to_le_bytes());
    chunk.extend_from_slice(b"pdta");
    chunk.extend_from_slice(&sub);
    chunk
}

fn write_sub_chunk(buf: &mut Vec<u8>, id: &[u8; 4], data: &[u8]) {
    buf.extend_from_slice(id);
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(data);
    // Pad to even boundary
    if data.len() % 2 != 0 {
        buf.push(0);
    }
}

fn write_sub_chunk_str(buf: &mut Vec<u8>, id: &[u8; 4], s: &str) {
    let mut data: Vec<u8> = s.bytes().take(255).collect();
    data.push(0); // null terminator
    write_sub_chunk(buf, id, &data);
}

fn write_phdr_record(
    buf: &mut Vec<u8>,
    name: &str,
    preset: u16,
    bank: u16,
    bag_idx: u16,
    _library: u32,
    _genre: u32,
) {
    // 38 bytes: 20-char name + preset(u16) + bank(u16) + bag_idx(u16) + library(u32) + genre(u32) + morphology(u32)
    let mut name_bytes = [0u8; 20];
    let src = name.as_bytes();
    let len = src.len().min(19);
    name_bytes[..len].copy_from_slice(&src[..len]);
    buf.extend_from_slice(&name_bytes);
    buf.extend_from_slice(&preset.to_le_bytes());
    buf.extend_from_slice(&bank.to_le_bytes());
    buf.extend_from_slice(&bag_idx.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes()); // library
    buf.extend_from_slice(&0u32.to_le_bytes()); // genre
    buf.extend_from_slice(&0u32.to_le_bytes()); // morphology
}

fn write_inst_record(buf: &mut Vec<u8>, name: &str, bag_idx: u16) {
    // 22 bytes: 20-char name + bag_idx(u16)
    let mut name_bytes = [0u8; 20];
    let src = name.as_bytes();
    let len = src.len().min(19);
    name_bytes[..len].copy_from_slice(&src[..len]);
    buf.extend_from_slice(&name_bytes);
    buf.extend_from_slice(&bag_idx.to_le_bytes());
}

fn write_bag_record(buf: &mut Vec<u8>, gen_idx: u16, mod_idx: u16) {
    buf.extend_from_slice(&gen_idx.to_le_bytes());
    buf.extend_from_slice(&mod_idx.to_le_bytes());
}

fn write_shdr_record(
    buf: &mut Vec<u8>,
    name: &str,
    start: u32,
    end: u32,
    loop_start: u32,
    loop_end: u32,
    sample_rate: u32,
    original_pitch: u8,
    pitch_correction: i8,
) {
    // 46 bytes: 20-char name + start(u32) + end(u32) + loop_start(u32) + loop_end(u32) +
    //           sample_rate(u32) + original_pitch(u8) + pitch_correction(i8) +
    //           sample_link(u16) + sample_type(u16)
    let mut name_bytes = [0u8; 20];
    let src = name.as_bytes();
    let len = src.len().min(19);
    name_bytes[..len].copy_from_slice(&src[..len]);
    buf.extend_from_slice(&name_bytes);
    buf.extend_from_slice(&start.to_le_bytes());
    buf.extend_from_slice(&end.to_le_bytes());
    buf.extend_from_slice(&loop_start.to_le_bytes());
    buf.extend_from_slice(&loop_end.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.push(original_pitch);
    buf.push(pitch_correction as u8);
    buf.extend_from_slice(&0u16.to_le_bytes()); // sample link
    buf.extend_from_slice(&1u16.to_le_bytes()); // sample type (mono)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Group, Zone};
    use std::path::PathBuf;

    #[test]
    fn sf2_starts_with_riff() {
        let inst = Instrument::builder("Test")
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

        let data = generate_sf2(&inst);
        assert_eq!(&data[..4], b"RIFF");
        assert_eq!(&data[8..12], b"sfbk");
    }

    #[test]
    fn sf2_export_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let inst = Instrument::builder("My SF2")
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

        Sf2Exporter.export(&inst, dir.path()).unwrap();
        let sf2_file = dir.path().join("My SF2").join("My SF2.sf2");
        assert!(sf2_file.exists());
        assert!(std::fs::metadata(&sf2_file).unwrap().len() > 100);
    }
}

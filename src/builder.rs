use std::collections::BTreeMap;
use std::path::Path;

use crate::naming::parse_sample_filename;

use crate::model::{Group, Instrument, Zone, compute_key_ranges};
use crate::traits::ExportError;

/// Build an Instrument model from a batch job output directory.
///
/// Expected layout:
/// ```text
/// output_dir/
///   <preset_name>/
///     <NoteName>_v<velocity>.wav
/// ```
///
/// For example: `Init/C4_v127.wav`, `Init/A2_v64.wav`
pub fn build_instrument_from_dir(
    output_dir: &Path,
    instrument_name: &str,
) -> Result<Instrument, ExportError> {
    let mut builder = Instrument::builder(instrument_name);
    let mut found_any = false;

    let dir_entries = std::fs::read_dir(output_dir)?;

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let preset_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let groups = build_groups_from_preset_dir(&path, &preset_name)?;
        for group in groups {
            if !group.zones.is_empty() {
                found_any = true;
            }
            builder = builder.group(group);
        }
    }

    if !found_any {
        return Err(ExportError::EmptyInstrument);
    }

    Ok(builder.build())
}

/// Parse WAV files in a preset directory into velocity-layer groups.
fn build_groups_from_preset_dir(
    preset_dir: &Path,
    preset_name: &str,
) -> Result<Vec<Group>, ExportError> {
    // Collect all samples: velocity -> Vec<(root_note, relative_path)>
    let mut velocity_map: BTreeMap<u8, Vec<(u8, std::path::PathBuf)>> = BTreeMap::new();

    let files = std::fs::read_dir(preset_dir)?;
    for file_entry in files {
        let file_entry = file_entry?;
        let file_path = file_entry.path();

        if file_path.extension().and_then(|e| e.to_str()) != Some("wav") {
            continue;
        }

        let filename = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if let Some((root_note, velocity)) = parse_sample_filename(&filename) {
            let relative_path = std::path::PathBuf::from(preset_name).join(&filename);
            velocity_map
                .entry(velocity)
                .or_default()
                .push((root_note, relative_path));
        }
    }

    if velocity_map.is_empty() {
        return Ok(vec![]);
    }

    // Compute velocity boundaries
    let velocities: Vec<u8> = velocity_map.keys().copied().collect();
    let vel_boundaries = compute_velocity_boundaries(&velocities);

    let mut groups = Vec::new();
    for (i, (&velocity, samples)) in velocity_map.iter().enumerate() {
        let (vel_low, vel_high) = vel_boundaries[i];

        // Sort by root note
        let mut samples = samples.clone();
        samples.sort_by_key(|(note, _)| *note);

        let root_notes: Vec<u8> = samples.iter().map(|(n, _)| *n).collect();
        let key_ranges = compute_key_ranges(&root_notes);

        let zones: Vec<Zone> = samples
            .iter()
            .zip(key_ranges.iter())
            .map(
                |((root_note, rel_path), &(key_low, _root, key_high))| Zone {
                    sample_path: rel_path.clone(),
                    root_note: *root_note,
                    key_low,
                    key_high,
                    velocity_low: vel_low,
                    velocity_high: vel_high,
                    gain_db: 0.0,
                    tune_cents: 0.0,
                    loop_info: None,
                    sample_start: 0,
                    sample_end: None,
                    round_robin: 0,
                },
            )
            .collect();

        let group_key_low = zones.iter().map(|z| z.key_low).min().unwrap_or(0);
        let group_key_high = zones.iter().map(|z| z.key_high).max().unwrap_or(127);

        groups.push(Group {
            name: format!("{preset_name} v{velocity}"),
            key_low: group_key_low,
            key_high: group_key_high,
            velocity_low: vel_low,
            velocity_high: vel_high,
            zones,
        });
    }

    Ok(groups)
}

/// Compute velocity boundaries for sorted velocity layers using midpoint splitting.
fn compute_velocity_boundaries(velocities: &[u8]) -> Vec<(u8, u8)> {
    if velocities.is_empty() {
        return vec![];
    }
    if velocities.len() == 1 {
        return vec![(1, 127)];
    }

    let mut boundaries = Vec::with_capacity(velocities.len());
    for (i, &vel) in velocities.iter().enumerate() {
        let low = if i == 0 {
            1
        } else {
            (velocities[i - 1] + vel) / 2 + 1
        };
        let high = if i == velocities.len() - 1 {
            127
        } else {
            (vel + velocities[i + 1]) / 2
        };
        let _ = vel; // used above in boundary computation
        boundaries.push((low, high));
    }
    boundaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Create a fake WAV file (just an empty file for filename parsing tests).
    fn create_fake_wav(dir: &Path, filename: &str) {
        std::fs::write(dir.join(filename), b"RIFF fake wav").unwrap();
    }

    #[test]
    fn parse_sample_filename_valid() {
        assert_eq!(parse_sample_filename("C4_v127.wav"), Some((60, 127)));
        assert_eq!(parse_sample_filename("F#5_v64.wav"), Some((78, 64)));
        assert_eq!(parse_sample_filename("A2_v32.wav"), Some((45, 32)));
    }

    #[test]
    fn parse_sample_filename_invalid() {
        assert_eq!(parse_sample_filename("not_a_note.wav"), None);
        assert_eq!(parse_sample_filename("C4.wav"), None);
        assert_eq!(parse_sample_filename("C4_v127.mp3"), None);
        assert_eq!(parse_sample_filename(""), None);
    }

    #[test]
    fn velocity_boundaries_single() {
        assert_eq!(compute_velocity_boundaries(&[127]), vec![(1, 127)]);
    }

    #[test]
    fn velocity_boundaries_two() {
        let bounds = compute_velocity_boundaries(&[64, 127]);
        assert_eq!(bounds, vec![(1, 95), (96, 127)]);
    }

    #[test]
    fn velocity_boundaries_four() {
        let bounds = compute_velocity_boundaries(&[32, 64, 96, 127]);
        assert_eq!(bounds.len(), 4);
        assert_eq!(bounds[0].0, 1);
        assert_eq!(bounds[3].1, 127);
        // No gaps
        for i in 1..bounds.len() {
            assert_eq!(bounds[i].0, bounds[i - 1].1 + 1);
        }
    }

    #[test]
    fn build_from_dir_simple() {
        let dir = tempdir().unwrap();
        let preset_dir = dir.path().join("Init");
        std::fs::create_dir(&preset_dir).unwrap();

        create_fake_wav(&preset_dir, "C2_v127.wav");
        create_fake_wav(&preset_dir, "C4_v127.wav");
        create_fake_wav(&preset_dir, "C6_v127.wav");

        let instrument = build_instrument_from_dir(dir.path(), "Test Synth").unwrap();
        assert_eq!(instrument.name, "Test Synth");
        assert_eq!(instrument.groups.len(), 1);
        assert_eq!(instrument.groups[0].zones.len(), 3);

        // Zones should be sorted by root note
        let zones = &instrument.groups[0].zones;
        assert_eq!(zones[0].root_note, 36); // C2
        assert_eq!(zones[1].root_note, 60); // C4
        assert_eq!(zones[2].root_note, 84); // C6
    }

    #[test]
    fn build_from_dir_multi_velocity() {
        let dir = tempdir().unwrap();
        let preset_dir = dir.path().join("Bass");
        std::fs::create_dir(&preset_dir).unwrap();

        create_fake_wav(&preset_dir, "C4_v64.wav");
        create_fake_wav(&preset_dir, "C4_v127.wav");
        create_fake_wav(&preset_dir, "E4_v64.wav");
        create_fake_wav(&preset_dir, "E4_v127.wav");

        let instrument = build_instrument_from_dir(dir.path(), "Bass").unwrap();
        assert_eq!(instrument.groups.len(), 2); // 2 velocity layers
        assert_eq!(instrument.total_zones(), 4);
    }

    #[test]
    fn build_from_empty_dir_fails() {
        let dir = tempdir().unwrap();
        let result = build_instrument_from_dir(dir.path(), "Empty");
        assert!(matches!(result, Err(ExportError::EmptyInstrument)));
    }
}

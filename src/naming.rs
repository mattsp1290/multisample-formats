/// Convert a MIDI note number (0-127) to a human-readable name like "C4" or "F#5".
pub fn note_number_to_name(note: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (note / 12) as i8 - 1;
    let name = NAMES[(note % 12) as usize];
    format!("{name}{octave}")
}

/// Parse a note name like "C4", "F#5", or "C-1" into a MIDI note number.
/// Returns `None` for an unrecognized name or a note outside 0–127.
pub fn note_name_to_number(name: &str) -> Option<u8> {
    if name.is_empty() {
        return None;
    }

    let bytes = name.as_bytes();
    let semitone: i32 = match bytes[0] {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };

    let rest = &name[1..];
    let (semitone, octave_str) = if let Some(octave) = rest.strip_prefix('#') {
        (semitone + 1, octave)
    } else {
        (semitone, rest)
    };

    let octave: i32 = octave_str.parse().ok()?;
    let note = (octave + 1) * 12 + semitone;

    if (0..=127).contains(&note) {
        Some(note as u8)
    } else {
        None
    }
}

/// Parse a sample filename like `"C4_v127.wav"` into `(root_note, velocity)`.
pub fn parse_sample_filename(filename: &str) -> Option<(u8, u8)> {
    let stem = filename.strip_suffix(".wav")?;
    let (note_str, vel_part) = stem.rsplit_once("_v")?;
    let root_note = note_name_to_number(note_str)?;
    let velocity: u8 = vel_part.parse().ok()?;
    Some((root_note, velocity))
}

/// Generate a rendered-sample filename, preserving the source naming convention.
/// The parser accepts only round_robin = 0 filenames.
pub fn render_sample_filename(note: u8, velocity: u8, round_robin: u8) -> String {
    let note_name = note_number_to_name(note);
    let ext = "wav";
    if round_robin > 0 {
        format!("{note_name}_v{velocity}_rr{round_robin}.{ext}")
    } else {
        format!("{note_name}_v{velocity}.{ext}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn note_name() {
        assert_eq!(note_number_to_name(60), "C4");
        assert_eq!(note_number_to_name(69), "A4");
        assert_eq!(note_number_to_name(0), "C-1");
    }
    #[test]
    fn note_name_parsing() {
        assert_eq!(note_name_to_number("C4"), Some(60));
        assert_eq!(note_name_to_number("A4"), Some(69));
        assert_eq!(note_name_to_number("C-1"), Some(0));
        assert_eq!(note_name_to_number("G9"), Some(127));
        assert_eq!(note_name_to_number("F#5"), Some(78));
        assert_eq!(note_name_to_number("D#3"), Some(51));
        assert_eq!(note_name_to_number(""), None);
        assert_eq!(note_name_to_number("X4"), None);
    }
    #[test]
    fn note_name_roundtrip() {
        for note in 0..=127u8 {
            assert_eq!(note_name_to_number(&note_number_to_name(note)), Some(note));
        }
    }
    #[test]
    fn rendered_filename_source_behavior() {
        for note in 0..=127u8 {
            for velocity in [1, 64, 127] {
                for rr in [0, 1, 2] {
                    let filename = render_sample_filename(note, velocity, rr);
                    let expected = if rr == 0 {
                        Some((note, velocity))
                    } else {
                        None
                    };
                    assert_eq!(parse_sample_filename(&filename), expected, "{filename}");
                }
            }
        }
    }
}

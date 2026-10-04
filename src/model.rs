use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Universal multisample instrument representation.
/// This is the format-agnostic data model that all exporters consume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    pub vendor: String,
    pub description: String,
    pub category: String,
    pub groups: Vec<Group>,
}

impl Instrument {
    pub fn builder(name: impl Into<String>) -> InstrumentBuilder {
        InstrumentBuilder {
            name: name.into(),
            vendor: String::new(),
            description: String::new(),
            category: String::new(),
            groups: Vec::new(),
        }
    }

    /// Total number of zones across all groups.
    pub fn total_zones(&self) -> usize {
        self.groups.iter().map(|g| g.zones.len()).sum()
    }
}

/// A group of zones (e.g., one velocity layer, or one round-robin variant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub key_low: u8,
    pub key_high: u8,
    pub velocity_low: u8,
    pub velocity_high: u8,
    pub zones: Vec<Zone>,
}

/// A single sample zone within a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    /// Path to the WAV sample file (relative to export root).
    pub sample_path: PathBuf,
    /// MIDI root note this sample was recorded at.
    pub root_note: u8,
    /// Key range this zone covers.
    pub key_low: u8,
    pub key_high: u8,
    /// Velocity range this zone covers.
    pub velocity_low: u8,
    pub velocity_high: u8,
    /// Gain adjustment in dB.
    pub gain_db: f64,
    /// Fine tuning in cents.
    pub tune_cents: f64,
    /// Loop information.
    pub loop_info: Option<LoopInfo>,
    /// Sample start position in frames (0 = file start).
    #[serde(default)]
    pub sample_start: u64,
    /// Sample end position in frames (None = file end).
    #[serde(default)]
    pub sample_end: Option<u64>,
    /// Round-robin group index (0 = no RR).
    pub round_robin: u8,
}

/// Loop point information for a sample.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LoopInfo {
    pub mode: LoopMode,
    /// Loop start in samples.
    pub start: u64,
    /// Loop end in samples.
    pub end: u64,
    /// Crossfade length in samples.
    pub crossfade: u64,
    /// Continue looping during release phase (Tonverk-specific).
    #[serde(default)]
    pub keep_looping_on_release: bool,
}

/// Loop playback modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    None,
    Forward,
    Backward,
    PingPong,
}

impl std::str::FromStr for LoopMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "None" => Ok(LoopMode::None),
            "Forward" => Ok(LoopMode::Forward),
            "Backward" => Ok(LoopMode::Backward),
            "PingPong" => Ok(LoopMode::PingPong),
            other => Err(format!("Unknown loop mode: {other}")),
        }
    }
}

/// Builder for constructing an Instrument step by step.
pub struct InstrumentBuilder {
    name: String,
    vendor: String,
    description: String,
    category: String,
    groups: Vec<Group>,
}

impl InstrumentBuilder {
    pub fn vendor(mut self, vendor: impl Into<String>) -> Self {
        self.vendor = vendor.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.category = cat.into();
        self
    }

    pub fn group(mut self, group: Group) -> Self {
        self.groups.push(group);
        self
    }

    pub fn build(self) -> Instrument {
        Instrument {
            name: self.name,
            vendor: self.vendor,
            description: self.description,
            category: self.category,
            groups: self.groups,
        }
    }
}

/// Helper to compute zone key ranges for evenly-spaced samples.
/// Given a sorted list of root notes, assigns each zone from
/// midpoint-below to midpoint-above.
pub fn compute_key_ranges(root_notes: &[u8]) -> Vec<(u8, u8, u8)> {
    if root_notes.is_empty() {
        return Vec::new();
    }
    if root_notes.len() == 1 {
        return vec![(0, root_notes[0], 127)];
    }

    let mut ranges = Vec::with_capacity(root_notes.len());
    for (i, &root) in root_notes.iter().enumerate() {
        let low = if i == 0 {
            0
        } else {
            (root_notes[i - 1] + root) / 2 + 1
        };
        let high = if i == root_notes.len() - 1 {
            127
        } else {
            (root + root_notes[i + 1]) / 2
        };
        ranges.push((low, root, high));
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instrument_builder() {
        let group = Group {
            name: "Layer 1".into(),
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
        };

        let inst = Instrument::builder("Test Piano")
            .vendor("Test")
            .category("Piano")
            .group(group)
            .build();

        assert_eq!(inst.name, "Test Piano");
        assert_eq!(inst.total_zones(), 1);
        assert_eq!(inst.groups.len(), 1);
    }

    #[test]
    fn key_range_computation() {
        // Single note
        let ranges = compute_key_ranges(&[60]);
        assert_eq!(ranges, vec![(0, 60, 127)]);

        // Multiple notes at interval 12
        let ranges = compute_key_ranges(&[48, 60, 72]);
        assert_eq!(ranges[0], (0, 48, 54));
        assert_eq!(ranges[1], (55, 60, 66));
        assert_eq!(ranges[2], (67, 72, 127));
    }

    #[test]
    fn key_range_empty() {
        assert!(compute_key_ranges(&[]).is_empty());
    }
}

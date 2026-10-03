/// All supported export formats.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    Sfz,
    DecentSampler,
    BitwigMultisample,
    Kontakt,
    SoundFont2,
    MpcKeygroup,
    KorgMultisample,
    AbletonSampler,
    ElektronTonverk,
    OpXy,
    WavBundle,
}

impl ExportFormat {
    /// File extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Sfz => "sfz",
            Self::DecentSampler => "dspreset",
            Self::BitwigMultisample => "multisample",
            Self::Kontakt => "nki",
            Self::SoundFont2 => "sf2",
            Self::MpcKeygroup => "xpm",
            Self::KorgMultisample => "korgmultisample",
            Self::AbletonSampler => "adv",
            Self::ElektronTonverk => "tonverk",
            Self::OpXy => "json",
            Self::WavBundle => "wav",
        }
    }

    /// Human-readable name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sfz => "SFZ",
            Self::DecentSampler => "DecentSampler",
            Self::BitwigMultisample => "Bitwig Multisample",
            Self::Kontakt => "NKI (Kontakt)",
            Self::SoundFont2 => "SoundFont 2",
            Self::MpcKeygroup => "MPC Keygroup",
            Self::KorgMultisample => "Korg Multisample",
            Self::AbletonSampler => "Ableton Sampler",
            Self::ElektronTonverk => "Elektron Tonverk",
            Self::OpXy => "Teenage Engineering OP-XY",
            Self::WavBundle => "WAV Bundle",
        }
    }

    /// Filesystem-friendly directory name for this format.
    pub fn dir_name(&self) -> &'static str {
        match self {
            Self::Sfz => "SFZ",
            Self::DecentSampler => "DecentSampler",
            Self::BitwigMultisample => "Bitwig",
            Self::Kontakt => "Kontakt",
            Self::SoundFont2 => "SF2",
            Self::MpcKeygroup => "MPC Keygroup",
            Self::KorgMultisample => "Korg",
            Self::AbletonSampler => "Ableton",
            Self::ElektronTonverk => "Tonverk",
            Self::OpXy => "OP-XY",
            Self::WavBundle => "WAV Bundle",
        }
    }

    /// All available formats.
    pub fn all() -> &'static [ExportFormat] {
        &[
            Self::Sfz,
            Self::DecentSampler,
            Self::BitwigMultisample,
            Self::Kontakt,
            Self::SoundFont2,
            Self::MpcKeygroup,
            Self::KorgMultisample,
            Self::AbletonSampler,
            Self::ElektronTonverk,
            Self::OpXy,
            Self::WavBundle,
        ]
    }
}

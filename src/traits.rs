use std::path::Path;

use crate::format::ExportFormat;

use crate::model::Instrument;

/// Error type for export operations.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("format error: {0}")]
    Format(String),

    #[error("no zones in instrument")]
    EmptyInstrument,

    #[error("missing sample file: {0}")]
    MissingSample(String),
}

/// Trait that each format exporter must implement.
pub trait FormatExporter: Send + Sync {
    /// The export format this exporter produces.
    fn format(&self) -> ExportFormat;

    /// Human-readable name of this exporter.
    fn name(&self) -> &str;

    /// Export the instrument to the output directory.
    /// The exporter creates format-specific files and references the WAV samples.
    fn export(&self, instrument: &Instrument, output_dir: &Path) -> Result<(), ExportError>;

    /// Name of the subdirectory inside the export directory where sample WAV
    /// files are placed. Return `None` for self-contained formats that embed
    /// audio data and need no external sample files (e.g., SF2).
    ///
    /// **Important:** The returned value must match the directory name used in
    /// the format's metadata files (e.g., Ableton ADG references `Samples/…`).
    fn samples_subdir(&self) -> Option<&'static str> {
        Some("samples")
    }

    /// Validate that the instrument can be exported in this format.
    /// Returns Ok(()) if valid, or an error describing the problem.
    fn validate(&self, instrument: &Instrument) -> Result<(), ExportError> {
        if instrument.total_zones() == 0 {
            return Err(ExportError::EmptyInstrument);
        }
        Ok(())
    }
}

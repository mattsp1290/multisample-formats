use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::format::ExportFormat;

use crate::model::Instrument;
use crate::traits::{ExportError, FormatExporter};
use crate::util::{copy_samples, sanitize_filename};

/// Registry of format exporters.
/// Allows registering exporters and exporting an instrument to one or more formats.
pub struct ExporterRegistry {
    exporters: HashMap<ExportFormat, Box<dyn FormatExporter>>,
}

impl ExporterRegistry {
    pub fn new() -> Self {
        Self {
            exporters: HashMap::new(),
        }
    }

    /// Register an exporter for a specific format.
    pub fn register(&mut self, exporter: Box<dyn FormatExporter>) {
        self.exporters.insert(exporter.format(), exporter);
    }

    /// Get an exporter by format.
    pub fn get(&self, format: ExportFormat) -> Option<&dyn FormatExporter> {
        self.exporters.get(&format).map(|e| e.as_ref())
    }

    /// List all registered formats.
    pub fn formats(&self) -> Vec<ExportFormat> {
        self.exporters.keys().copied().collect()
    }

    /// Export an instrument to a single format.
    ///
    /// - `source_dir`: directory containing the rendered WAV samples
    /// - `exports_base`: base directory for exports (e.g., `.../Multisamples/exports/`)
    ///
    /// Creates `exports_base/<format_dir_name>/` and delegates to the exporter,
    /// then copies samples into the instrument's export directory.
    /// Returns the path to the instrument's export directory.
    #[tracing::instrument(name = "export_single", skip_all, fields(format = ?format))]
    pub fn export_single(
        &self,
        instrument: &Instrument,
        format: ExportFormat,
        source_dir: &Path,
        exports_base: &Path,
    ) -> Result<PathBuf, ExportError> {
        let exporter = self.exporters.get(&format).ok_or_else(|| {
            ExportError::Format(format!("no exporter for {}", format.display_name()))
        })?;
        exporter.validate(instrument)?;

        let format_dir = exports_base.join(format.dir_name());
        exporter.export(instrument, &format_dir)?;

        let instrument_dir = format_dir.join(sanitize_filename(&instrument.name));
        if let Some(subdir) = exporter.samples_subdir() {
            copy_samples(instrument, source_dir, &instrument_dir, subdir)?;
        }

        Ok(instrument_dir)
    }

    /// Export an instrument to multiple formats.
    /// Returns a map of format -> result (instrument export directory path) for each format.
    #[tracing::instrument(name = "export_multi", skip_all, fields(num_formats = formats.len()))]
    pub fn export_multi(
        &self,
        instrument: &Instrument,
        formats: &[ExportFormat],
        source_dir: &Path,
        exports_base: &Path,
    ) -> HashMap<ExportFormat, Result<PathBuf, ExportError>> {
        formats
            .iter()
            .map(|&fmt| {
                let result = self.export_single(instrument, fmt, source_dir, exports_base);
                (fmt, result)
            })
            .collect()
    }
}

impl Default for ExporterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A no-op test exporter.
    struct NoopExporter;

    impl FormatExporter for NoopExporter {
        fn format(&self) -> ExportFormat {
            ExportFormat::Sfz
        }
        fn name(&self) -> &str {
            "SFZ (test)"
        }
        fn export(&self, _instrument: &Instrument, _output_dir: &Path) -> Result<(), ExportError> {
            Ok(())
        }
    }

    #[test]
    fn register_and_lookup() {
        let mut reg = ExporterRegistry::new();
        reg.register(Box::new(NoopExporter));

        assert!(reg.get(ExportFormat::Sfz).is_some());
        assert!(reg.get(ExportFormat::DecentSampler).is_none());
        assert_eq!(reg.formats().len(), 1);
    }

    #[test]
    fn export_unregistered_format_errors() {
        let reg = ExporterRegistry::new();
        let inst = Instrument::builder("Test").build();
        let result = reg.export_single(
            &inst,
            ExportFormat::Sfz,
            Path::new("/tmp"),
            Path::new("/tmp/exports"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn export_empty_instrument_fails_validation() {
        let mut reg = ExporterRegistry::new();
        reg.register(Box::new(NoopExporter));

        let inst = Instrument::builder("Empty").build();
        let result = reg.export_single(
            &inst,
            ExportFormat::Sfz,
            Path::new("/tmp"),
            Path::new("/tmp/exports"),
        );
        assert!(matches!(result, Err(ExportError::EmptyInstrument)));
    }
}

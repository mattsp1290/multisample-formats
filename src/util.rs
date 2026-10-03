use std::path::{Path, PathBuf};

/// Sanitize a string for use in filenames.
/// Replaces problematic characters with underscores.
pub fn sanitize_filename(name: &str) -> String {
    let result: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .trim_start_matches('.')
        .to_string();

    // Prefix Windows reserved names to avoid filesystem errors on Windows.
    if is_windows_reserved(&result) {
        format!("_{result}")
    } else {
        result
    }
}

/// Check if a name matches a Windows reserved device name (case-insensitive).
fn is_windows_reserved(name: &str) -> bool {
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let upper = name.trim().to_uppercase();
    RESERVED.contains(&upper.as_str())
}

/// Build a sample filename from components.
pub fn sample_filename(
    preset_name: &str,
    note_name: &str,
    velocity: u8,
    round_robin: u8,
) -> String {
    let mut name = format!(
        "{}_{}_v{}",
        sanitize_filename(preset_name),
        note_name,
        velocity
    );
    if round_robin > 0 {
        name.push_str(&format!("_rr{}", round_robin));
    }
    name.push_str(".wav");
    name
}

/// Create output directory structure for an export.
pub fn prepare_output_dir(base: &Path, instrument_name: &str) -> std::io::Result<PathBuf> {
    let dir = base.join(sanitize_filename(instrument_name));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Copy all WAV samples referenced by `instrument` from `source_root`
/// into `export_dir/<samples_subdir>/<zone.sample_path>`.
///
/// Creates intermediate directories as needed.
/// Skips files that already exist at the destination (idempotent).
/// Each unique `zone.sample_path` is processed once.
pub fn copy_samples(
    instrument: &crate::model::Instrument,
    source_root: &Path,
    export_dir: &Path,
    samples_subdir: &str,
) -> std::io::Result<()> {
    // Canonicalize source_root once so all per-zone checks are fast comparisons.
    let canonical_source_root = source_root.canonicalize()?;

    let mut seen = std::collections::HashSet::new();

    for group in &instrument.groups {
        for zone in &group.zones {
            if !seen.insert(zone.sample_path.clone()) {
                continue;
            }

            // Reject absolute paths and any path containing `..` components
            // before joining onto export_dir, preventing writes outside it.
            if zone.sample_path.is_absolute() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "sample_path must be relative, got: {}",
                        zone.sample_path.display()
                    ),
                ));
            }
            if zone
                .sample_path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "sample_path must not contain `..` components, got: {}",
                        zone.sample_path.display()
                    ),
                ));
            }

            let src = source_root.join(&zone.sample_path);
            let dst = export_dir.join(samples_subdir).join(&zone.sample_path);

            if dst.exists() {
                continue;
            }

            // Canonicalize src to resolve symlinks and verify it stays within
            // source_root.  We do this after the dst-exists check so that a
            // missing file surfaces as a NotFound error below (not a
            // canonicalize failure) when the file doesn't exist at all.
            match src.canonicalize() {
                Ok(canonical_src) => {
                    if !canonical_src.starts_with(&canonical_source_root) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            format!(
                                "sample path escapes source root: {} -> {}",
                                zone.sample_path.display(),
                                canonical_src.display()
                            ),
                        ));
                    }

                    if let Some(parent) = dst.parent() {
                        std::fs::create_dir_all(parent)?;
                    }

                    match std::fs::copy(&canonical_src, &dst) {
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            tracing::warn!(src = %src.display(), "sample file missing, skipping");
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tracing::warn!(src = %src.display(), "sample file missing, skipping");
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_basic() {
        assert_eq!(sanitize_filename("Hello World"), "Hello World");
        assert_eq!(sanitize_filename("foo/bar:baz"), "foo_bar_baz");
        assert_eq!(sanitize_filename("a*b?c"), "a_b_c");
    }

    #[test]
    fn sanitize_strips_leading_dots() {
        assert_eq!(sanitize_filename(".."), "");
        assert_eq!(sanitize_filename("."), "");
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("my.instrument"), "my.instrument");
    }

    #[test]
    fn sanitize_strips_traversal_with_slashes() {
        // Slashes become underscores, then leading dots are stripped
        assert_eq!(sanitize_filename("../../etc"), "_.._etc");
        assert_eq!(sanitize_filename("foo/../bar"), "foo_.._bar");
    }

    #[test]
    fn sanitize_windows_reserved_names() {
        assert_eq!(sanitize_filename("CON"), "_CON");
        assert_eq!(sanitize_filename("con"), "_con");
        assert_eq!(sanitize_filename("AUX"), "_AUX");
        assert_eq!(sanitize_filename("NUL"), "_NUL");
        assert_eq!(sanitize_filename("COM1"), "_COM1");
        assert_eq!(sanitize_filename("LPT9"), "_LPT9");
        // Not reserved
        assert_eq!(sanitize_filename("CONSOLE"), "CONSOLE");
        assert_eq!(sanitize_filename("auxiliary"), "auxiliary");
    }

    #[test]
    fn sample_filename_format() {
        assert_eq!(
            sample_filename("Init Patch", "C4", 127, 0),
            "Init Patch_C4_v127.wav"
        );
        assert_eq!(sample_filename("Bass", "A2", 64, 1), "Bass_A2_v64_rr1.wav");
    }

    #[test]
    fn copy_samples_basic() {
        use crate::model::{Group, Instrument, Zone};

        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source");
        let export = tmp.path().join("export");
        std::fs::create_dir_all(source.join("Init")).unwrap();
        std::fs::write(source.join("Init/C4_v127.wav"), b"RIFF fake wav").unwrap();

        let inst = Instrument::builder("Test")
            .group(Group {
                name: "Init v127".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![Zone {
                    sample_path: PathBuf::from("Init/C4_v127.wav"),
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

        copy_samples(&inst, &source, &export, "samples").unwrap();

        let dst = export.join("samples/Init/C4_v127.wav");
        assert!(dst.exists());
        assert_eq!(std::fs::read(&dst).unwrap(), b"RIFF fake wav");

        // Idempotent: calling again should not error
        copy_samples(&inst, &source, &export, "samples").unwrap();
    }

    /// Helper that builds a single-zone `Instrument` with the given sample path.
    fn inst_with_path(sample_path: PathBuf) -> crate::model::Instrument {
        use crate::model::{Group, Instrument, Zone};
        Instrument::builder("Test")
            .group(Group {
                name: "g".into(),
                key_low: 0,
                key_high: 127,
                velocity_low: 0,
                velocity_high: 127,
                zones: vec![Zone {
                    sample_path,
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

    #[test]
    fn copy_samples_rejects_dotdot_component() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source");
        let export = tmp.path().join("export");
        std::fs::create_dir_all(&source).unwrap();

        let inst = inst_with_path(PathBuf::from("../escape.wav"));
        let err = copy_samples(&inst, &source, &export, "samples").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn copy_samples_rejects_absolute_sample_path() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source");
        let export = tmp.path().join("export");
        std::fs::create_dir_all(&source).unwrap();

        let inst = inst_with_path(PathBuf::from("/etc/passwd"));
        let err = copy_samples(&inst, &source, &export, "samples").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn copy_samples_rejects_symlink_escape() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source");
        let export = tmp.path().join("export");
        let outside = tmp.path().join("outside");
        std::fs::create_dir_all(&source).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.wav"), b"secret").unwrap();

        // Create a symlink inside source that points outside source_root.
        std::os::unix::fs::symlink(&outside, source.join("link")).unwrap();

        let inst = inst_with_path(PathBuf::from("link/secret.wav"));
        let err = copy_samples(&inst, &source, &export, "samples").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }
}

#[path = "support/fixture.rs"]
mod fixture;

use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read, path::Path};

// Recorded from the unmodified source exporters at a220778, never regenerated
// from this crate. Comparing the whole path set catches added/missing outputs.
const BASELINE: &str = include_str!("../docs/golden-baseline.json");

fn collect_files(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_files(root, &path, files);
        } else {
            files.insert(
                path.strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/"),
                std::fs::read(path).unwrap(),
            );
        }
    }
}

#[test]
fn all_exporters_match_source_revision() {
    let tmp = tempfile::tempdir().unwrap();
    fixture::export_fixture(tmp.path());
    let root = tmp.path().join("exports");
    let mut actual = BTreeMap::new();
    collect_files(&root, &root, &mut actual);
    let baseline: serde_json::Value = serde_json::from_str(BASELINE).unwrap();
    let expected = baseline["files"].as_array().unwrap();
    assert_eq!(actual.len(), expected.len(), "output file set changed");
    for file in expected {
        let path = file["path"].as_str().unwrap();
        let bytes = actual
            .remove(path)
            .unwrap_or_else(|| panic!("missing {path}"));
        let content = if file["comparison"] == "sha256-decompressed-gzip" {
            let mut content = Vec::new();
            flate2::read::GzDecoder::new(bytes.as_slice())
                .read_to_end(&mut content)
                .unwrap();
            content
        } else {
            bytes
        };
        assert_eq!(
            format!("{:x}", Sha256::digest(content)),
            file["sha256"].as_str().unwrap(),
            "{path}"
        );
    }
    assert!(actual.is_empty());
}

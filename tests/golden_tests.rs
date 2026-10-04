#[path = "support/fixture.rs"]
mod fixture;

use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read, path::Path};

// Recorded from the unmodified source exporters at a220778, never regenerated
// from this crate. Comparing the whole path set catches added/missing outputs.
const BASELINE: &str = include_str!("../docs/golden-baseline.json");

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Baseline {
    source_revision: String,
    flate2: String,
    exporter_count: usize,
    not_byte_comparable: Vec<String>,
    files: Vec<BaselineFile>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BaselineFile {
    path: String,
    comparison: Comparison,
    sha256: String,
}

#[derive(serde::Deserialize)]
enum Comparison {
    #[serde(rename = "sha256")]
    Sha256,
    #[serde(rename = "sha256-decompressed-gzip")]
    Sha256DecompressedGzip,
}

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
    let baseline: Baseline = serde_json::from_str(BASELINE).unwrap();
    assert_eq!(
        baseline.source_revision,
        "a2207788282cf943d9d12cfeab63b4897af808b2"
    );
    assert_eq!(baseline.flate2, "1.1.9");
    assert_eq!(baseline.exporter_count, 16);
    assert!(baseline.not_byte_comparable.is_empty());
    let expected = baseline.files;
    assert_eq!(actual.len(), expected.len(), "output file set changed");
    for file in expected {
        let path = file.path;
        let bytes = actual
            .remove(&path)
            .unwrap_or_else(|| panic!("missing {path}"));
        let content = match file.comparison {
            Comparison::Sha256 => bytes,
            Comparison::Sha256DecompressedGzip => {
                let mut content = Vec::new();
                flate2::read::GzDecoder::new(bytes.as_slice())
                    .read_to_end(&mut content)
                    .unwrap();
                content
            }
        };
        assert_eq!(
            format!("{:x}", Sha256::digest(content)),
            file.sha256,
            "{path}"
        );
    }
    assert!(actual.is_empty());
}

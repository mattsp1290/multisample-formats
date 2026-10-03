#![cfg(feature = "serde")]
use multisample_formats::ExportFormat;

#[test]
fn serialized_variant_names_match_source_contract() {
    let names = [
        "Sfz",
        "DecentSampler",
        "BitwigMultisample",
        "Kontakt",
        "SoundFont2",
        "MpcKeygroup",
        "KorgMultisample",
        "AbletonSampler",
        "ElektronTonverk",
        "OpXy",
        "WavBundle",
    ];
    assert_eq!(ExportFormat::all().len(), names.len());
    for (&format, name) in ExportFormat::all().iter().zip(names) {
        let json = format!("\"{name}\"");
        assert_eq!(serde_json::to_string(&format).unwrap(), json);
        assert_eq!(serde_json::from_str::<ExportFormat>(&json).unwrap(), format);
    }
}

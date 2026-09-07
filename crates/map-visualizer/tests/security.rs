// SPDX-License-Identifier: MIT

use map_visualizer::{
    http::{Rejection, parse},
    json::decode,
    storage::{ArtifactRoot, File, valid_bundle_id},
};
use proptest::prelude::*;

#[test]
fn json_rejects_duplicate_keys_invalid_unicode_depth_and_oversized_input() {
    for bytes in [
        br#"{"a":1,"a":2}"#.as_slice(),
        br#"{"a":{"x":1,"x":2}}"#,
        br#"{"x":"\ud800"}"#,
        br#"{"x":1}{}"#,
    ] {
        assert!(decode(bytes, 1024).is_err());
    }
    assert!(decode(&[b' '; 1025], 1024).is_err());
    let nested = format!("{}0{}", "[".repeat(26), "]".repeat(26));
    assert!(decode(nested.as_bytes(), 1024).is_err());
    assert!(decode(br#"{"label":"<script>data only</script>"}"#, 1024).is_ok());
}

#[test]
fn origin_host_path_and_method_are_fenced() {
    assert!(
        parse(
            b"GET /api/current HTTP/1.1\r\nHost: 127.0.0.1:3847\r\n\r\n",
            3847
        )
        .is_ok()
    );
    for path in [
        "/../secret",
        "/%2e%2e/secret",
        "//example.com/x",
        "http://example.com/",
        "/api/frame/a?url=x",
        "/x\\y",
    ] {
        let request = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:3847\r\n\r\n");
        // Double slash is rejected by the fixed router rather than generic path grammar.
        if !path.starts_with("//") {
            assert!(parse(request.as_bytes(), 3847).is_err(), "{path}");
        }
    }
    for header in [
        "Host: attacker.example:3847",
        "Host: 127.0.0.1:3847\r\nOrigin: https://attacker.example",
        "Host: 127.0.0.1:3847\r\nSec-Fetch-Site: cross-site",
    ] {
        assert_eq!(
            parse(
                format!("GET /api/current HTTP/1.1\r\n{header}\r\n\r\n").as_bytes(),
                3847
            ),
            Err(Rejection::Forbidden)
        );
    }
    assert_eq!(
        parse(
            b"POST /api/current HTTP/1.1\r\nHost: 127.0.0.1:3847\r\n\r\n",
            3847
        ),
        Err(Rejection::MethodNotAllowed)
    );
    assert!(
        parse(
            b"GET / HTTP/1.1\r\nHost: 127.0.0.1:3847\r\nHost: attacker\r\n\r\n",
            3847
        )
        .is_err()
    );
    assert!(
        parse(
            b"GET / HTTP/1.1\r\nHost: 127.0.0.1:3847\r\nTransfer-Encoding: chunked\r\n\r\n",
            3847
        )
        .is_err()
    );
}

#[test]
fn file_reads_are_bounded_to_typed_artifacts_and_safe_bundle_names() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("manifest.json"), b"{}").unwrap();
    let root = ArtifactRoot::open(temp.path()).unwrap();
    assert_eq!(root.read(File::Manifest).unwrap(), b"{}");
    for id in ["..", "../other", "/etc", "a/b", "a%2fb", "", "C:\\x"] {
        assert!(!valid_bundle_id(id));
        assert!(root.bundle(id).is_err());
    }
    std::fs::write(temp.path().join("manifest.json"), vec![b'x'; 65537]).unwrap();
    assert!(root.read(File::Manifest).is_err());
}

#[test]
#[cfg(unix)]
fn symlink_files_and_bundle_escapes_are_rejected() {
    use std::os::unix::fs::symlink;
    let rootdir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("manifest.json"), b"private").unwrap();
    symlink(
        outside.path().join("manifest.json"),
        rootdir.path().join("manifest.json"),
    )
    .unwrap();
    symlink(outside.path(), rootdir.path().join("escape")).unwrap();
    let root = ArtifactRoot::open(rootdir.path()).unwrap();
    assert!(root.read(File::Manifest).is_err());
    assert!(root.bundle("escape").is_err());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn bounded_fuzz_does_not_panic_or_admit_nonlocal_authority(bytes in prop::collection::vec(any::<u8>(),0..10000)) {
        if let Ok(request)=parse(&bytes,3847) {
            prop_assert!(request.path.starts_with('/'));
            prop_assert!(!request.path.contains('%'));
            prop_assert!(!request.path.contains(".."));
        }
        let _=decode(&bytes,8192);
    }
    #[test]
    fn duplicate_keys_always_fail(key in "[a-z]{1,24}") {
        let bytes=format!("{{\"{key}\":0,\"{key}\":1}}");
        prop_assert!(decode(bytes.as_bytes(),1024).is_err());
    }
}

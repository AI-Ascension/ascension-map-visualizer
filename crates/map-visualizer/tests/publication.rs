// SPDX-License-Identifier: MIT
use map_visualizer::publication::publish;
use std::sync::{Arc, Barrier};

#[test]
fn concurrent_publications_commit_one_complete_directory_without_overwrite() {
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("output");
    let barrier = Arc::new(Barrier::new(3));
    let threads: Vec<_> = [b"first".as_slice(), b"second".as_slice()]
        .into_iter()
        .map(|bytes| {
            let output = output.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                publish(
                    &output,
                    &[("manifest.json", bytes), ("analysis.json", bytes)],
                )
            })
        })
        .collect();
    barrier.wait();
    let successes = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .filter(Result::is_ok)
        .count();
    assert_eq!(successes, 1);
    let manifest = std::fs::read(output.join("manifest.json")).unwrap();
    let analysis = std::fs::read(output.join("analysis.json")).unwrap();
    assert_eq!(manifest, analysis);
    assert!(manifest == b"first" || manifest == b"second");
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
}

#[test]
fn rejected_publication_preserves_existing_files_and_rejects_paths() {
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("existing");
    std::fs::create_dir(&output).unwrap();
    std::fs::write(output.join("valued.txt"), b"preserve").unwrap();
    assert!(publish(&output, &[("manifest.json", b"replace")]).is_err());
    assert_eq!(
        std::fs::read(output.join("valued.txt")).unwrap(),
        b"preserve"
    );
    assert!(publish(&root.path().join("new"), &[("../escape", b"bad")]).is_err());
    assert!(!root.path().join("new").exists());
    assert!(!root.path().join("escape").exists());
}

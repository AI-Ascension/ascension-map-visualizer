# Independent native packaging review

Reviewed source: `5b1d196480685a313bc2417d5b4450a63cc89ce5`.

Disposition: passed for the native packaging privacy boundary. This review establishes source, archive, and native executable behavior; it does not establish live host/provider acceptance or release publication.

An independent reviewer freshly extracted both archives listed in [native packaging evidence](native-packaging.json), verified every checksum (252 Linux and 254 Windows entries), and rejected unsafe archive-member paths. All-file UTF-8 and UTF-16LE scans found no local build-path leak in either repaired package. The superseded `0fb4670` Windows archive failed the same scan, confirming the negative control detects the original defect.

Both packaged binaries passed native `doctor`, `demo`, and `validate` runs. Each fresh demo contained 76 nodes, 182 edges, and one visible unreachable node. All ten generated files matched across platforms. The canonical digest of the relative-file/hash set was `e97227d4f5eb19f773de2b7ada5e5a50ec2202372844a35a1a06e01cf5ca49d7`.

Source probes verified inherited compiler flags, remap ordering, debug stripping, a relative Windows PDB reference, and packaging into an output path containing spaces. Linux debug sections were absent. The reviewer independently confirmed all five hosted jobs passed at [the exact source revision](https://github.com/AI-Ascension/ascension-map-visualizer/actions/runs/34092979262).

The evidence applies to the recorded native environments. Arbitrary Windows aliases (8.3, junction, subst, UNC, and case variants) and `target-dir` from an external Cargo configuration were not exhaustively characterized. No leak was observed in the tested environments; custom environments require their own artifact scan.

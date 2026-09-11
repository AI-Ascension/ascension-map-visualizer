// SPDX-License-Identifier: MIT
use map_visualizer::arguments::{Command, Input, parse};
use std::ffi::OsString;

fn args(values: &[&str]) -> Result<Command, &'static str> {
    parse(values.iter().map(OsString::from))
}

#[test]
fn reject_ambiguous_inputs_and_unbounded_images() {
    for values in [
        vec!["serve", "--bundle", "one", "--root", "two"],
        vec![
            "render", "--bundle", "one", "--out", "two", "--width", "8192", "--height", "16384",
        ],
        vec!["serve", "--root", "one", "--port", "65536"],
        vec!["validate", "--bundle", "one", "--bundle", "two"],
        vec!["doctor", "--root", "one"],
        vec!["render", "--bundle", "--out", "two"],
        vec!["replay", "--bundle", "one"],
        vec!["serve", "--root", "one", "--host", "0.0.0.0"],
    ] {
        assert!(args(&values).is_err(), "accepted {values:?}");
    }
}

#[test]
fn replay_is_historical_and_ephemeral_port_is_allowed() {
    assert_eq!(
        args(&["replay", "--root", "frames", "--port", "0"]).unwrap(),
        Command::Serve {
            input: Input::Root("frames".into()),
            port: 0,
            historical: true
        }
    );
    assert!(matches!(
        args(&["demo", "--out", "new folder", "--serve"]).unwrap(),
        Command::Demo { serve: true, .. }
    ));
}

#[cfg(unix)]
#[test]
fn filesystem_paths_do_not_require_utf8() {
    use std::os::unix::ffi::OsStringExt;
    let path = OsString::from_vec(vec![b'.', b'/', 255]);
    let result = parse([
        OsString::from("validate"),
        OsString::from("--bundle"),
        path.clone(),
    ])
    .unwrap();
    assert_eq!(
        result,
        Command::Validate {
            bundle: path.into()
        }
    );
}

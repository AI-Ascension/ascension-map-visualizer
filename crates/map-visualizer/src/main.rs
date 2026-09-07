// SPDX-License-Identifier: MIT
use map_visualizer::{arguments, cli};
use std::process::ExitCode;

fn main() -> ExitCode {
    let result = arguments::parse(std::env::args_os().skip(1))
        .map_err(|message| cli::Failure { code: 2, message })
        .and_then(cli::execute);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("map-visualizer: {}", error.message);
            ExitCode::from(error.code)
        }
    }
}

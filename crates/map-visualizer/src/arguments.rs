// SPDX-License-Identifier: MIT
//! Finite CLI grammar; paths stay opaque OS strings.

use crate::presentation::Settings;
use std::ffi::OsString;
use std::path::PathBuf;

pub const HELP: &str = "map-visualizer validate --bundle DIR\nmap-visualizer render --bundle DIR --out NEWDIR [--width N --height N]\nmap-visualizer serve (--bundle DIR | --root DIR) [--port N]\nmap-visualizer replay --root DIR [--port N]\nmap-visualizer demo --out NEWDIR [--serve] [--port N]\nmap-visualizer doctor\n\nExit codes: 0 success, 2 invalid arguments, 3 rejected artifact, 4 I/O or rendering failure.\nListeners bind only to 127.0.0.1; port 0 requests an available port.\n";

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Help,
    Doctor,
    Validate {
        bundle: PathBuf,
    },
    Render {
        bundle: PathBuf,
        out: PathBuf,
        settings: Settings,
    },
    Serve {
        input: Input,
        port: u16,
        historical: bool,
    },
    Demo {
        out: PathBuf,
        serve: bool,
        port: u16,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum Input {
    Bundle(PathBuf),
    Root(PathBuf),
}

pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, &'static str> {
    let mut args = arguments.into_iter();
    let command = args.next().ok_or("a command is required")?;
    let command = command.to_str().ok_or("unknown command")?;
    if !matches!(
        command,
        "help" | "--help" | "-h" | "doctor" | "validate" | "render" | "serve" | "replay" | "demo"
    ) {
        return Err("unknown command");
    }
    let mut bundle = None;
    let mut root = None;
    let mut out = None;
    let mut width = None;
    let mut height = None;
    let mut port = None;
    let mut serve = false;
    let mut count = 0;
    while let Some(flag) = args.next() {
        count += 1;
        if count > 8 {
            return Err("too many arguments");
        }
        let flag = flag.to_str().ok_or("unknown flag")?;
        match flag {
            "--serve" if command == "demo" && !serve => serve = true,
            "--bundle" if matches!(command, "validate" | "render" | "serve") => {
                set_path(&mut bundle, args.next())?
            }
            "--root" if matches!(command, "serve" | "replay") => set_path(&mut root, args.next())?,
            "--out" if matches!(command, "render" | "demo") => set_path(&mut out, args.next())?,
            "--width" if command == "render" => set_number(&mut width, args.next())?,
            "--height" if command == "render" => set_number(&mut height, args.next())?,
            "--port" if matches!(command, "serve" | "replay" | "demo") => {
                set_number(&mut port, args.next())?
            }
            _ => return Err("unknown, repeated, or inapplicable flag"),
        }
    }
    let port = u16::try_from(port.unwrap_or(0)).map_err(|_| "invalid port")?;
    match command {
        "help" | "--help" | "-h" => Ok(Command::Help),
        "doctor" => Ok(Command::Doctor),
        "validate" => Ok(Command::Validate {
            bundle: bundle.ok_or("--bundle is required")?,
        }),
        "render" => {
            let defaults = Settings::default();
            let settings = Settings {
                width: width.unwrap_or(defaults.width),
                height: height.unwrap_or(defaults.height),
            };
            settings
                .validate()
                .map_err(|_| "invalid image dimensions")?;
            Ok(Command::Render {
                bundle: bundle.ok_or("--bundle is required")?,
                out: out.ok_or("--out is required")?,
                settings,
            })
        }
        "serve" | "replay" => {
            let input = match (bundle, root) {
                (Some(path), None) => Input::Bundle(path),
                (None, Some(path)) => Input::Root(path),
                _ => return Err("exactly one of --bundle or --root is required"),
            };
            Ok(Command::Serve {
                input,
                port,
                historical: command == "replay",
            })
        }
        "demo" => Ok(Command::Demo {
            out: out.ok_or("--out is required")?,
            serve,
            port,
        }),
        _ => Err("unknown command"),
    }
}

fn set_path(target: &mut Option<PathBuf>, value: Option<OsString>) -> Result<(), &'static str> {
    if target.is_some() {
        return Err("repeated flag");
    }
    let value = value.ok_or("missing flag value")?;
    if value.is_empty() || value.to_str().is_some_and(|s| s.starts_with("--")) {
        return Err("missing path");
    }
    *target = Some(PathBuf::from(value));
    Ok(())
}

fn set_number(target: &mut Option<u32>, value: Option<OsString>) -> Result<(), &'static str> {
    if target.is_some() {
        return Err("repeated flag");
    }
    let value = value.ok_or("missing flag value")?;
    let value = value.to_str().ok_or("invalid number")?;
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid number");
    }
    *target = Some(value.parse().map_err(|_| "number exceeds limit")?);
    Ok(())
}

// SPDX-License-Identifier: MIT
//! Read-only loopback server with four bounded in-flight requests and fixed routes.

use crate::{http, storage::valid_bundle_id};
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub trait Source: Send + Sync + 'static {
    fn current(&self) -> Result<Vec<u8>, &'static str>;
    fn replay(&self) -> Result<Vec<u8>, &'static str>;
    fn frame(&self, id: &str) -> Result<Vec<u8>, &'static str>;
}

pub struct Assets {
    pub html: Vec<u8>,
    pub script: Vec<u8>,
    pub style: Vec<u8>,
}

pub struct Server {
    listener: TcpListener,
    port: u16,
}

impl Server {
    pub fn bind(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        Ok(Self { listener, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn run(
        self,
        source: Arc<dyn Source>,
        assets: Arc<Assets>,
        stop: Arc<AtomicBool>,
    ) -> io::Result<()> {
        let mut workers: Vec<JoinHandle<()>> = Vec::new();
        let mut failure = None;
        'serving: while !stop.load(Ordering::Acquire) {
            let mut index = 0;
            while index < workers.len() {
                if workers[index].is_finished() {
                    if workers.swap_remove(index).join().is_err() {
                        failure = Some(io::Error::other("viewer worker failed"));
                        break 'serving;
                    }
                } else {
                    index += 1;
                }
            }
            match self.listener.accept() {
                Ok((mut socket, address)) => {
                    if !address.ip().is_loopback() || workers.len() >= 4 {
                        let _ = http::respond(
                            &mut socket,
                            503,
                            "text/plain; charset=utf-8",
                            b"viewer busy",
                            false,
                        );
                        continue;
                    }
                    let source = Arc::clone(&source);
                    let assets = Arc::clone(&assets);
                    let port = self.port;
                    workers.push(thread::spawn(move || {
                        handle(socket, port, &*source, &assets)
                    }));
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
        }
        for worker in workers {
            if worker.join().is_err() {
                failure = Some(io::Error::other("viewer worker failed"));
            }
        }
        failure.map_or(Ok(()), Err)
    }
}

fn handle(mut socket: TcpStream, port: u16, source: &dyn Source, assets: &Assets) {
    let request = match http::read_request(&mut socket, port) {
        Ok(request) => request,
        Err(rejection) => {
            let _ = http::respond(
                &mut socket,
                rejection.status(),
                "text/plain; charset=utf-8",
                b"request rejected",
                false,
            );
            return;
        }
    };
    let static_asset = match request.path.as_str() {
        "/" | "/index.html" => Some(("text/html; charset=utf-8", assets.html.as_slice())),
        "/app.js" => Some(("text/javascript; charset=utf-8", assets.script.as_slice())),
        "/style.css" => Some(("text/css; charset=utf-8", assets.style.as_slice())),
        _ => None,
    };
    if let Some((kind, bytes)) = static_asset {
        let _ = http::respond(&mut socket, 200, kind, bytes, request.head);
        return;
    }
    let result = match request.path.as_str() {
        "/api/current" => source.current(),
        "/api/replay" => source.replay(),
        path if path.starts_with("/api/frame/") && valid_bundle_id(&path[11..]) => {
            source.frame(&path[11..])
        }
        _ => {
            let _ = http::respond(
                &mut socket,
                404,
                "text/plain; charset=utf-8",
                b"not found",
                request.head,
            );
            return;
        }
    };
    match result {
        Ok(bytes) if bytes.len() <= 2 * 1024 * 1024 => {
            let _ = http::respond(&mut socket, 200, "application/json", &bytes, request.head);
        }
        _ => {
            let _ = http::respond(
                &mut socket,
                503,
                "text/plain; charset=utf-8",
                b"complete validated bundle unavailable",
                request.head,
            );
        }
    }
}

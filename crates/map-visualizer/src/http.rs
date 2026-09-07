// SPDX-License-Identifier: MIT
//! Finite loopback HTTP parsing and response generation, without arbitrary URL fetching.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub const MAX_HEADERS: usize = 8192;
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'";

#[derive(Debug, Eq, PartialEq)]
pub struct Request {
    pub path: String,
    pub head: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rejection {
    BadRequest,
    Forbidden,
    MethodNotAllowed,
    TooLarge,
}

pub fn parse(bytes: &[u8], port: u16) -> Result<Request, Rejection> {
    if bytes.len() > MAX_HEADERS {
        return Err(Rejection::TooLarge);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Rejection::BadRequest)?;
    if !text.ends_with("\r\n\r\n") {
        return Err(Rejection::BadRequest);
    }
    let mut lines = text[..text.len() - 4].split("\r\n");
    let start: Vec<_> = lines
        .next()
        .ok_or(Rejection::BadRequest)?
        .split(' ')
        .collect();
    if start.len() != 3 || start[2] != "HTTP/1.1" {
        return Err(Rejection::BadRequest);
    }
    if start[0] != "GET" && start[0] != "HEAD" {
        return Err(Rejection::MethodNotAllowed);
    }
    if start[1].len() > 256
        || !start[1].starts_with('/')
        || start[1].contains("..")
        || start[1]
            .bytes()
            .any(|b| !b.is_ascii_alphanumeric() && !b"/._-".contains(&b))
    {
        return Err(Rejection::BadRequest);
    }
    let mut headers = std::collections::BTreeMap::new();
    for line in lines {
        let (key, value) = line.split_once(':').ok_or(Rejection::BadRequest)?;
        if key.is_empty()
            || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || value.bytes().any(|b| b < 32 && b != b'\t' || b == 127)
            || headers
                .insert(key.to_ascii_lowercase(), value.trim())
                .is_some()
        {
            return Err(Rejection::BadRequest);
        }
    }
    let host = headers.get("host").ok_or(Rejection::BadRequest)?;
    if ![
        format!("127.0.0.1:{port}"),
        format!("localhost:{port}"),
        format!("[::1]:{port}"),
    ]
    .iter()
    .any(|h| h == host)
    {
        return Err(Rejection::Forbidden);
    }
    if headers
        .get("origin")
        .is_some_and(|origin| *origin != format!("http://{host}"))
        || headers
            .get("sec-fetch-site")
            .is_some_and(|s| !["same-origin", "none"].contains(s))
    {
        return Err(Rejection::Forbidden);
    }
    if headers.contains_key("transfer-encoding")
        || headers.get("content-length").is_some_and(|v| *v != "0")
    {
        return Err(Rejection::BadRequest);
    }
    Ok(Request {
        path: start[1].to_owned(),
        head: start[0] == "HEAD",
    })
}

pub fn read_request(stream: &mut TcpStream, port: u16) -> Result<Request, Rejection> {
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .map_err(|_| Rejection::BadRequest)?;
    let start = Instant::now();
    let mut bytes = Vec::with_capacity(1024);
    let mut block = [0u8; 512];
    loop {
        if start.elapsed() > Duration::from_secs(1) {
            return Err(Rejection::BadRequest);
        }
        let length = stream.read(&mut block).map_err(|_| Rejection::BadRequest)?;
        if length == 0 {
            return Err(Rejection::BadRequest);
        }
        if bytes.len() + length > MAX_HEADERS {
            return Err(Rejection::TooLarge);
        }
        bytes.extend_from_slice(&block[..length]);
        if bytes.windows(4).any(|w| w == b"\r\n\r\n") {
            return parse(&bytes, port);
        }
    }
}

pub fn respond(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    head: bool,
) -> io::Result<()> {
    if !matches!(
        content_type,
        "text/plain; charset=utf-8"
            | "text/html; charset=utf-8"
            | "text/javascript; charset=utf-8"
            | "text/css; charset=utf-8"
            | "application/json"
    ) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsupported response MIME type",
        ));
    }
    stream.set_write_timeout(Some(Duration::from_millis(500)))?;
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Content Too Large",
        503 => "Service Unavailable",
        _ => "Internal Server Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nContent-Security-Policy: {CONTENT_SECURITY_POLICY}\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nCross-Origin-Resource-Policy: same-origin\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    if !head {
        let start = Instant::now();
        let mut remaining = body;
        while !remaining.is_empty() {
            let budget = Duration::from_secs(2)
                .checked_sub(start.elapsed())
                .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "response deadline"))?;
            stream.set_write_timeout(Some(budget.min(Duration::from_millis(500))))?;
            let count = stream.write(&remaining[..remaining.len().min(64 * 1024)])?;
            if count == 0 {
                return Err(io::Error::new(io::ErrorKind::WriteZero, "response closed"));
            }
            remaining = &remaining[count..];
        }
    }
    Ok(())
}

impl Rejection {
    pub fn status(self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::MethodNotAllowed => 405,
            Self::TooLarge => 413,
        }
    }
}

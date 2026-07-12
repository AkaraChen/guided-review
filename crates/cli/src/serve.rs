use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

/// Serve `dir` read-only over HTTP on 127.0.0.1 until the process is stopped.
pub fn serve(dir: &Path, port: u16) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("cannot serve {}", dir.display()))?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("failed to bind 127.0.0.1:{port}"))?;
    let address = listener.local_addr().context("no local address")?;
    println!("http://{address}/");

    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        if let Err(error) = respond(&mut stream, &dir) {
            eprintln!("request failed: {error:#}");
        }
    }
    Ok(())
}

fn respond(stream: &mut TcpStream, dir: &Path) -> Result<()> {
    let mut request_line = String::new();
    BufReader::new(&mut *stream)
        .read_line(&mut request_line)
        .context("failed to read request line")?;
    let mut parts = request_line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));

    if method != "GET" {
        return write_response(stream, "405 Method Not Allowed", "text/plain", b"GET only");
    }

    let body = resolve(dir, target).and_then(|path| fs::read(&path).ok());
    match body {
        Some(body) => write_response(stream, "200 OK", content_type(target), &body),
        None => write_response(stream, "404 Not Found", "text/plain", b"not found"),
    }
}

/// Maps a request target to a file under `dir`; directories fall back to
/// their index.html. Rejects every dot segment so requests cannot escape
/// `dir` (encoded forms like %2e stay literal because targets are not
/// percent-decoded).
fn resolve(dir: &Path, target: &str) -> Option<PathBuf> {
    let path = target.split(['?', '#']).next().unwrap_or("");
    let mut resolved = dir.to_path_buf();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        if segment.starts_with('.') || segment.contains('\\') {
            return None;
        }
        resolved.push(segment);
    }
    if resolved.is_dir() {
        resolved.push("index.html");
    }
    Some(resolved)
}

fn content_type(target: &str) -> &'static str {
    let path = target.split(['?', '#']).next().unwrap_or("");
    match path.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        _ => "text/html; charset=utf-8",
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::resolve;

    #[test]
    fn rejects_dot_segments_that_would_escape_the_served_directory() {
        let dir = Path::new("/served");

        assert_eq!(resolve(dir, "/../secret"), None);
        assert_eq!(resolve(dir, "/nested/../../secret"), None);
        assert_eq!(resolve(dir, "/.hidden"), None);
    }

    #[test]
    fn maps_plain_targets_below_the_served_directory() {
        let dir = Path::new("/served");

        assert_eq!(
            resolve(dir, "/owner-repo-1.html?live=1"),
            Some(dir.join("owner-repo-1.html"))
        );
    }
}

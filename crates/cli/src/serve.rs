use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

/// How long after a stop beacon the server waits for a follow-up request.
/// Navigating between pages fires `pagehide` too, so the next page's GET
/// must arrive within this window to cancel the shutdown.
const STOP_GRACE: Duration = Duration::from_secs(2);

/// Injected into every served HTML page so a closing tab tells the server
/// to shut down.
const STOP_BEACON: &str =
    "<script>addEventListener(\"pagehide\",()=>navigator.sendBeacon(\"/stop\"));</script>";

enum Handled {
    Request,
    Stop,
}

/// Serve `dir` read-only over HTTP on 127.0.0.1 until the process is stopped
/// or a served page's closing tab posts to `/stop`.
pub fn serve(dir: &Path, port: u16) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("cannot serve {}", dir.display()))?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("failed to bind 127.0.0.1:{port}"))?;
    listener
        .set_nonblocking(true)
        .context("failed to poll the listener")?;
    let address = listener.local_addr().context("no local address")?;
    println!("http://{address}/");

    let mut stop_at: Option<Instant> = None;
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_nonblocking(false).ok();
                match respond(&mut stream, &dir) {
                    Ok(Handled::Stop) => stop_at = Some(Instant::now() + STOP_GRACE),
                    Ok(Handled::Request) => stop_at = None,
                    Err(error) => eprintln!("request failed: {error:#}"),
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if stop_at.is_some_and(|deadline| Instant::now() >= deadline) {
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => continue,
        }
    }
}

fn respond(stream: &mut TcpStream, dir: &Path) -> Result<Handled> {
    let mut request_line = String::new();
    BufReader::new(&mut *stream)
        .read_line(&mut request_line)
        .context("failed to read request line")?;
    let mut parts = request_line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));

    if method == "POST" && path_of(target) == "/stop" {
        write_response(stream, "204 No Content", "text/plain", b"")?;
        return Ok(Handled::Stop);
    }
    if method != "GET" {
        write_response(stream, "405 Method Not Allowed", "text/plain", b"GET only")?;
        return Ok(Handled::Request);
    }

    let body = resolve(dir, target).and_then(|path| fs::read(&path).ok());
    match body {
        Some(body) => {
            let content_type = content_type(target);
            let body = if content_type.starts_with("text/html") {
                with_stop_beacon(body)
            } else {
                body
            };
            write_response(stream, "200 OK", content_type, &body)?;
        }
        None => write_response(stream, "404 Not Found", "text/plain", b"not found")?,
    }
    Ok(Handled::Request)
}

/// Appends the stop beacon script, before `</body>` when present so the
/// page stays well-formed.
fn with_stop_beacon(mut body: Vec<u8>) -> Vec<u8> {
    let closing = b"</body>";
    let insert_at = body
        .windows(closing.len())
        .rposition(|window| window.eq_ignore_ascii_case(closing))
        .unwrap_or(body.len());
    body.splice(insert_at..insert_at, STOP_BEACON.bytes());
    body
}

/// Maps a request target to a file under `dir`; directories fall back to
/// their index.html. Rejects every dot segment so requests cannot escape
/// `dir` (encoded forms like %2e stay literal because targets are not
/// percent-decoded).
fn resolve(dir: &Path, target: &str) -> Option<PathBuf> {
    let path = path_of(target);
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

fn path_of(target: &str) -> &str {
    target.split(['?', '#']).next().unwrap_or("")
}

fn content_type(target: &str) -> &'static str {
    match path_of(target).rsplit('.').next() {
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

    use super::{STOP_BEACON, resolve, with_stop_beacon};

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

    #[test]
    fn injects_the_stop_beacon_before_the_closing_body_tag() {
        let page = b"<html><body><p>review</p></body></html>".to_vec();

        let served = String::from_utf8(with_stop_beacon(page)).unwrap();

        let expected = format!("<html><body><p>review</p>{STOP_BEACON}</body></html>");
        assert_eq!(served, expected);
    }

    #[test]
    fn appends_the_stop_beacon_when_no_closing_body_tag_exists() {
        let page = b"<p>fragment</p>".to_vec();

        let served = String::from_utf8(with_stop_beacon(page)).unwrap();

        assert_eq!(served, format!("<p>fragment</p>{STOP_BEACON}"));
    }
}

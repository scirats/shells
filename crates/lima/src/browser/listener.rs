use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::opener::Opener;

const MAX_REQUEST_BYTES: usize = 16 * 1024;
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const ACCEPT_POLL: Duration = Duration::from_millis(100);

pub struct Listener {
    port: u16,
    shutdown: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Listener {
    pub fn bind(opener: Arc<dyn Opener>) -> std::io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker = worker(listener, opener, shutdown.clone());
        Ok(Self {
            port,
            shutdown,
            worker: Some(worker),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn worker(
    listener: TcpListener,
    opener: Arc<dyn Opener>,
    shutdown: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        while !shutdown.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _ = serve(stream, opener.as_ref());
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(ACCEPT_POLL);
                }
                Err(_) => break,
            }
        }
    })
}

fn serve(mut stream: TcpStream, opener: &dyn Opener) -> std::io::Result<()> {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let Some(url) = request_url(&mut stream)? else {
        return respond(&mut stream, 400, "only http and https URLs are supported");
    };
    if opener.open(&url).is_err() {
        return respond(&mut stream, 502, "the host cannot open the URL");
    }
    respond(&mut stream, 204, "")
}

fn request_url(stream: &mut TcpStream) -> std::io::Result<Option<String>> {
    let request = read_request(stream)?;
    let body = body(&request).unwrap_or_default();
    let url = String::from_utf8_lossy(body).trim().to_string();
    Ok(is_openable(&url).then_some(url))
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        if buffer.len() > MAX_REQUEST_BYTES {
            break;
        }
        if let Some(body) = body(&buffer) {
            if body.len() >= content_length(&buffer) {
                break;
            }
        }
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
    Ok(buffer)
}

fn body(request: &[u8]) -> Option<&[u8]> {
    let start = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")?
        + 4;
    request.get(start..)
}

fn content_length(request: &[u8]) -> usize {
    for line in String::from_utf8_lossy(request).lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                return value.trim().parse().unwrap_or(0);
            }
        }
    }
    0
}

fn is_openable(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

fn respond(stream: &mut TcpStream, code: u16, message: &str) -> std::io::Result<()> {
    let reason = match code {
        204 => "No Content",
        400 => "Bad Request",
        502 => "Bad Gateway",
        _ => "Internal Server Error",
    };
    let response = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{message}",
        message.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingOpener {
        urls: Mutex<Vec<String>>,
    }

    impl Opener for RecordingOpener {
        fn open(&self, url: &str) -> std::io::Result<()> {
            self.urls.lock().unwrap().push(url.to_string());
            Ok(())
        }
    }

    fn call(listener: &Listener, body: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", listener.port())).unwrap();
        stream.set_read_timeout(Some(READ_TIMEOUT)).unwrap();
        let request = format!(
            "POST /open HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    #[test]
    fn forwards_an_http_url_to_the_opener() {
        let opener = Arc::new(RecordingOpener::default());
        let listener = Listener::bind(opener.clone()).unwrap();

        let response = call(&listener, "https://example.com/login");

        assert!(response.starts_with("HTTP/1.1 204"));
        assert_eq!(
            opener.urls.lock().unwrap().as_slice(),
            ["https://example.com/login"]
        );
    }

    #[test]
    fn rejects_a_non_http_url_without_opening_it() {
        let opener = Arc::new(RecordingOpener::default());
        let listener = Listener::bind(opener.clone()).unwrap();

        let response = call(&listener, "file:///etc/passwd");

        assert!(response.starts_with("HTTP/1.1 400"));
        assert!(opener.urls.lock().unwrap().is_empty());
    }
}

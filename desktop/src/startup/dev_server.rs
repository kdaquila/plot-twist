//! Development builds load the frontend from the dev server that `npm run tauri dev` starts.
//! When a development build is started without it (e.g. `cargo run`), or another program
//! holds its port, show a message naming the right command instead of the WebView's error
//! page or the other program's page. Prod builds skip this entirely.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use tauri::{App, Manager, Url};

const CONNECT_TIMEOUT: Duration = Duration::from_millis(300);
const READ_TIMEOUT: Duration = Duration::from_secs(2);
/// The frontend's `<title>` (ui/index.html): how the dev server is recognized as ours.
const OUR_TITLE: &str = "<title>plot-twist</title>";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DevServer {
    /// plot-twist's dev server answered.
    Ours,
    /// Nothing is listening at the dev-server address.
    Missing,
    /// Something else is listening there.
    Other,
}

/// In a development build, replace the page with a message if the dev server is not ours.
pub fn check(app: &App) {
    if !tauri::is_dev() {
        return;
    }
    let Some(dev_url) = app.config().build.dev_url.clone() else {
        return;
    };
    if !is_loopback(&dev_url) {
        return;
    }
    let state = probe(&dev_url);
    tracing::info!(%dev_url, ?state, "dev server check");
    let page = match state {
        DevServer::Ours => return,
        DevServer::Missing => missing_page(),
        DevServer::Other => other_page(dev_url.port_or_known_default().unwrap_or_default()),
    };
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let data_url = format!("data:text/html;charset=utf-8,{}", percent_encode(&page));
    match Url::parse(&data_url) {
        Ok(url) => {
            if let Err(error) = window.navigate(url) {
                tracing::error!(%error, "could not show the dev-server message");
            }
        }
        Err(error) => tracing::error!(%error, "could not build the dev-server message"),
    }
}

fn is_loopback(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

fn probe(url: &Url) -> DevServer {
    let (Some(host), Some(port)) = (url.host_str(), url.port_or_known_default()) else {
        return DevServer::Missing;
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let Ok(addrs) = (host, port).to_socket_addrs() else {
        return DevServer::Missing;
    };
    // `localhost` may resolve to both IPv6 and IPv4; the dev server may listen on either.
    let mut state = DevServer::Missing;
    for addr in addrs {
        match fetch_root(&addr, host, port) {
            None => {}
            Some(page) if page.contains(OUR_TITLE) => return DevServer::Ours,
            Some(_) => state = DevServer::Other,
        }
    }
    state
}

/// The reply to `GET /`, or `None` if nothing accepts a connection at `addr`.
fn fetch_root(addr: &SocketAddr, host: &str, port: u16) -> Option<String> {
    let mut stream = TcpStream::connect_timeout(addr, CONNECT_TIMEOUT).ok()?;
    let mut reply = Vec::new();
    let request = format!("GET / HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    // A server that accepts but misbehaves is still "something else": keep what was read.
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    if stream.write_all(request.as_bytes()).is_ok() {
        let _ = stream.read_to_end(&mut reply);
    }
    Some(String::from_utf8_lossy(&reply).into_owned())
}

fn missing_page() -> String {
    message_page(
        "The dev server is not running",
        "<p>This is a development build. It shows the app from the frontend dev server, which \
         only starts when you run the app in dev mode.</p>\
         <p>Close this window and start the app from the repository root with:</p>\
         <p><code>npm run tauri dev</code></p>\
         <p>To run the app without a dev server, build it in prod mode instead: see the \
         README.</p>",
    )
}

fn other_page(port: u16) -> String {
    message_page(
        &format!("Port {port} is used by another program"),
        &format!(
            "<p>This is a development build. It shows the app from the frontend dev server on \
             port {port}, but another program is using that port.</p>\
             <p>Close this window and the other program, then start the app from the \
             repository root with:</p>\
             <p><code>npm run tauri dev</code></p>"
        ),
    )
}

fn message_page(heading: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>plot-twist</title>
<style>
  :root {{ color-scheme: light dark; font-family: system-ui, sans-serif; }}
  body {{ margin: 0; min-height: 100vh; display: grid; place-items: center; }}
  main {{ max-width: 36rem; padding: 2rem; line-height: 1.5; }}
  code {{ font-size: 1.05em; padding: 0.1em 0.35em; border-radius: 4px; background: rgb(127 127 127 / 0.2); }}
</style>
</head>
<body>
<main>
<h1>{heading}</h1>
{body}
</main>
</body>
</html>
"#
    )
}

fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

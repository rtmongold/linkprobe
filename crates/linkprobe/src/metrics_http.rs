use std::sync::{Arc, RwLock};
use std::thread;

use tiny_http::{Header, Response, Server};

const METRICS_CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

pub type MetricsBody = Arc<RwLock<String>>;

pub fn shared_body(initial: String) -> MetricsBody {
    Arc::new(RwLock::new(initial))
}

pub fn spawn_metrics_server(listen: &str, body: MetricsBody) -> Result<(), String> {
    let listen = listen.to_string();
    thread::Builder::new()
        .name("linkprobe-metrics".into())
        .spawn(move || {
            let server = match Server::http(&listen) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: metrics listen on {listen}: {e}");
                    return;
                }
            };
            eprintln!("linkprobe: serving GET /metrics on http://{listen}/metrics");
            for request in server.incoming_requests() {
                let path = request.url().split('?').next().unwrap_or("/");
                let response = if path == "/metrics" {
                    let text = body.read().map(|g| g.clone()).unwrap_or_default();
                    Response::from_string(text).with_header(
                        Header::from_bytes(b"Content-Type", METRICS_CONTENT_TYPE.as_bytes())
                            .expect("content-type header"),
                    )
                } else {
                    Response::from_string("not found\n").with_status_code(404)
                };
                let _ = request.respond(response);
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

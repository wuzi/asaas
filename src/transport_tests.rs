use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;

fn client() -> crate::Client {
    crate::ClientBuilder::new()
        .api_key("fixture-key")
        .user_agent("sdk-test")
        .build()
        .unwrap()
}

// Keep accepting after the first response so a redirect or replay becomes observable.
async fn request_with_response(
    response: Option<&'static str>,
) -> (Result<reqwest::Response, reqwest::Error>, usize) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/original", listener.local_addr().unwrap());
    let done = Arc::new(AtomicBool::new(false));
    let count = Arc::new(AtomicUsize::new(0));
    let server_done = done.clone();
    let server_count = count.clone();
    let task = thread::spawn(move || {
        while !server_done.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut socket, _)) => {
                    socket
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut buffer = [0; 4096];
                    assert!(socket.read(&mut buffer).unwrap() > 0);
                    let attempt = server_count.fetch_add(1, Ordering::SeqCst);
                    if let Some(response) = response {
                        let response = if attempt == 0 {
                            response
                        } else {
                            "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        };
                        socket.write_all(response.as_bytes()).unwrap();
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(1))
                }
                Err(error) => panic!("fixture listener: {error}"),
            }
        }
    });
    let result = client().http.post(url).body("mutation").send().await;
    done.store(true, Ordering::SeqCst);
    task.join().unwrap();
    (result, count.load(Ordering::SeqCst))
}

#[tokio::test]
async fn default_transport_returns_redirect_without_following_it() {
    let (result, count) = request_with_response(Some("HTTP/1.1 307 Temporary Redirect\r\nLocation: /redirected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")).await;
    assert_eq!(
        result.unwrap().status(),
        reqwest::StatusCode::TEMPORARY_REDIRECT
    );
    assert_eq!(count, 1);
}

#[tokio::test]
async fn default_transport_does_not_replay_a_dropped_mutation_response() {
    let (result, count) = request_with_response(None).await;
    assert!(result.is_err());
    assert_eq!(count, 1);
}

#[tokio::test]
async fn default_transport_returns_rate_limit_without_retrying() {
    let (result, count) = request_with_response(Some("HTTP/1.1 429 Too Many Requests\r\nRetry-After: 0\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")).await;
    assert_eq!(
        result.unwrap().status(),
        reqwest::StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(count, 1);
}

// A local server that accepts TCP but never finishes the requested protocol phase.
async fn stalled_request(scheme: &str, hold: Duration) -> reqwest::Error {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("{scheme}://{}/stalled", listener.local_addr().unwrap());
    let task = thread::spawn(move || {
        let (socket, _) = listener.accept().unwrap();
        thread::sleep(hold);
        drop(socket);
    });
    let error = client().http.get(url).send().await.unwrap_err();
    task.join().unwrap();
    error
}

#[tokio::test]
async fn default_transport_bounds_total_request_duration() {
    let error = stalled_request("http", Duration::from_secs(21)).await;
    assert!(error.is_timeout(), "expected request timeout, got {error}");
}

#[tokio::test]
async fn default_transport_bounds_tls_connection_duration() {
    let error = stalled_request("https", Duration::from_secs(6)).await;
    assert!(error.is_timeout(), "expected connect timeout, got {error}");
}

#[tokio::test]
async fn sdk_transport_does_not_retry_http2_refused_stream() {
    // REFUSED_STREAM is a safe protocol NACK, but an automatic extra attempt
    // would still be invisible to the caller's native request budget.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/mutation", listener.local_addr().unwrap());
    let attempts = Arc::new(AtomicUsize::new(0));
    let server_attempts = attempts.clone();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let mut connection = h2::server::handshake(socket).await.unwrap();
        while let Some(stream) = connection.accept().await {
            let (_, mut response) = stream.unwrap();
            server_attempts.fetch_add(1, Ordering::SeqCst);
            response.send_reset(h2::Reason::REFUSED_STREAM);
        }
    });
    let http = crate::client::http_client_builder()
        .http2_prior_knowledge()
        .build()
        .unwrap();
    let result = http.post(url).body("mutation").send().await;
    server.abort();
    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
}

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct StubResponse {
    pub status: &'static str,
    pub headers: &'static [(&'static str, &'static str)],
    pub body: &'static str,
}

impl StubResponse {
    pub const fn json(body: &'static str) -> Self {
        Self {
            status: "200 OK",
            headers: &[],
            body,
        }
    }
}

pub async fn serve_once(
    method: &'static str,
    target: &'static str,
    response: StubResponse,
) -> (asaas::Client, tokio::task::JoinHandle<()>) {
    let cert = include_bytes!("../fixtures/loopback-only-cert.pem");
    let key = include_bytes!("../fixtures/loopback-only-key.pem");
    let identity = native_tls::Identity::from_pkcs8(cert, key).unwrap();
    let tls = tokio_native_tls::TlsAcceptor::from(native_tls::TlsAcceptor::new(identity).unwrap());
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let http = reqwest::Client::builder()
        .resolve("api-sandbox.asaas.com", listener.local_addr().unwrap())
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .https_only(true)
        .timeout(std::time::Duration::from_secs(5))
        .add_root_certificate(reqwest::Certificate::from_pem(cert).unwrap())
        .build()
        .unwrap();
    let client = asaas::ClientBuilder::new()
        .api_key("test-token")
        .user_agent("sdk-test")
        .http_client(http)
        .build()
        .unwrap();
    let task = tokio::spawn(async move {
        let (socket, peer) = listener.accept().await.unwrap();
        assert!(peer.ip().is_loopback());
        let mut stream = BufReader::new(tls.accept(socket).await.unwrap());
        let mut line = String::new();
        stream.read_line(&mut line).await.unwrap();
        assert_eq!(line, format!("{method} {target} HTTP/1.1\r\n"));

        let mut request_headers = String::new();
        loop {
            line.clear();
            stream.read_line(&mut line).await.unwrap();
            if line == "\r\n" {
                break;
            }
            request_headers.push_str(&line.to_ascii_lowercase());
        }
        assert!(request_headers.contains("access_token: test-token\r\n"));
        assert!(!request_headers.contains("content-type:"));
        assert!(!request_headers.contains("content-length:"));

        let mut response_headers = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
            response.status,
            response.body.len()
        );
        for (name, value) in response.headers {
            response_headers.push_str(name);
            response_headers.push_str(": ");
            response_headers.push_str(value);
            response_headers.push_str("\r\n");
        }
        response_headers.push_str("\r\n");
        stream.write_all(response_headers.as_bytes()).await.unwrap();
        stream.write_all(response.body.as_bytes()).await.unwrap();
    });
    (client, task)
}

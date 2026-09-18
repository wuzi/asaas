//! Fixtures follow the public full-payment and wallet schemas, fetched 2026-09-18.
//! https://docs.asaas.com/reference/recuperar-uma-unica-cobranca
//! https://docs.asaas.com/reference/recuperar-walletid
use asaas::ClientBuilder;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

async fn serve(
    body: &'static str,
    path: &'static str,
) -> (asaas::Client, tokio::task::JoinHandle<()>) {
    let cert = include_bytes!("fixtures/loopback-only-cert.pem");
    let key = include_bytes!("fixtures/loopback-only-key.pem");
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
    let client = ClientBuilder::new()
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
        assert_eq!(line, format!("GET {path} HTTP/1.1\r\n"));
        let mut headers = String::new();
        loop {
            line.clear();
            stream.read_line(&mut line).await.unwrap();
            if line == "\r\n" {
                break;
            }
            headers.push_str(&line.to_ascii_lowercase());
        }
        assert!(headers.contains("access_token: test-token\r\n"));
        assert!(!headers.contains("content-type:"));
        assert!(!headers.contains("content-length:"));
        stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    });
    (client, task)
}

#[tokio::test]
async fn full_payment_read_preserves_exact_numbers_and_unknown_native_values() {
    let (client, server) = serve(r#"{"id":"pay_123","status":"FUTURE_STATUS","billingType":"FUTURE_TYPE","value":9007199254740993.01,"originalValue":129.900000000000000001,"paymentDate":"2026-09-17","installment":"ins_123","installmentNumber":2}"#, "/v3/payments/pay_123").await;
    let payment = client.get_payment("pay_123").await.unwrap();
    assert_eq!(payment.value.unwrap().to_string(), "9007199254740993.01");
    assert_eq!(
        payment.original_value.unwrap().to_string(),
        "129.900000000000000001"
    );
    assert_eq!(payment.status, "FUTURE_STATUS");
    assert_eq!(payment.billing_type, "FUTURE_TYPE");
    assert_eq!(payment.installment.as_deref(), Some("ins_123"));
    server.await.unwrap();
}

#[tokio::test]
async fn wallet_read_uses_authenticated_public_endpoint() {
    let (client, server) = serve(r#"{"object":"list","hasMore":false,"totalCount":1,"limit":10,"offset":0,"data":[{"object":"wallet","id":"wallet-123"}]}"#, "/v3/wallets/").await;
    let wallets = client.get_wallets().await.unwrap();
    assert_eq!(wallets.data[0].id, "wallet-123");
    assert!(!wallets.has_more);
    server.await.unwrap();
}

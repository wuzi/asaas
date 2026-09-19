mod common;

use asaas::Error;
use common::{StubResponse, serve_once, serve_raw_once};

const PDF_PATH: &str = "/v3/installments/ins_123/paymentBook";
const LIMIT: usize = 10 * 1024 * 1024;

fn pdf(length: usize) -> Vec<u8> {
    let mut bytes = vec![b' '; length];
    bytes[..8].copy_from_slice(b"%PDF-1.7");
    bytes[length - 5..].copy_from_slice(b"%%EOF");
    bytes
}

fn response(body: &[u8], chunked: bool, status: &str) -> Vec<u8> {
    let framing = if chunked {
        "Transfer-Encoding: chunked\r\n".to_owned()
    } else {
        format!("Content-Length: {}\r\n", body.len())
    };
    let mut wire = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/pdf\r\n{framing}Connection: close\r\n\r\n"
    )
    .into_bytes();
    if chunked {
        for chunk in body.chunks(64 * 1024) {
            wire.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
            wire.extend_from_slice(chunk);
            wire.extend_from_slice(b"\r\n");
        }
        wire.extend_from_slice(b"0\r\n\r\n");
    } else {
        wire.extend_from_slice(body);
    }
    wire
}

#[tokio::test]
async fn payment_book_accepts_valid_pdf_at_limit_but_rejects_declared_and_streamed_overflow() {
    let headers_only = format!("HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", LIMIT + 1).into_bytes();
    let (client, server) = serve_raw_once("GET", PDF_PATH, headers_only).await;
    let early = client.get_installment_payment_book_pdf("ins_123").await;
    server.await.unwrap();
    let mut rejected = vec![matches!(early, Err(Error::RequestFailed { .. }))];
    for chunked in [false, true] {
        let valid = pdf(LIMIT);
        let (client, server) =
            serve_raw_once("GET", PDF_PATH, response(&valid, chunked, "200 OK")).await;
        let received = client
            .get_installment_payment_book_pdf("ins_123")
            .await
            .unwrap();
        assert!(
            received == valid,
            "ordinary PDF bytes must be preserved at the exact size limit"
        );
        server.await.unwrap();

        let (client, server) = serve_raw_once(
            "GET",
            PDF_PATH,
            response(&pdf(LIMIT + 1), chunked, "200 OK"),
        )
        .await;
        let result = client.get_installment_payment_book_pdf("ins_123").await;
        server.await.unwrap();
        rejected.push(matches!(result, Err(Error::RequestFailed { .. })));
    }
    assert_eq!(
        rejected,
        [true, true, true],
        "declared limit must reject before the body; streamed limit must also apply"
    );
}

#[tokio::test]
async fn payment_book_rejects_non_pdf_success_and_preserves_bounded_provider_errors() {
    for body in [
        b"<html>provider error</html>".as_slice(),
        b"%PDF-1.7 truncated",
    ] {
        let (client, server) =
            serve_raw_once("GET", PDF_PATH, response(body, false, "200 OK")).await;
        assert!(
            client
                .get_installment_payment_book_pdf("ins_123")
                .await
                .is_err()
        );
        server.await.unwrap();
    }
    let (client, server) = serve_once("GET", PDF_PATH, StubResponse::json("%PDF-1.7\n%%EOF")).await;
    assert!(
        client
            .get_installment_payment_book_pdf("ins_123")
            .await
            .is_err()
    );
    server.await.unwrap();
    let mut rejected =
        StubResponse::json(r#"{"errors":[{"code":"invalid_action","description":"Unavailable"}]}"#);
    rejected.status = "400 Bad Request";
    let (client, server) = serve_once("GET", PDF_PATH, rejected).await;
    let error = client
        .get_installment_payment_book_pdf("ins_123")
        .await
        .unwrap_err();
    assert_eq!(error.api_errors().unwrap()[0].code, "invalid_action");
    server.await.unwrap();

    let wire = response(&pdf(LIMIT + 1), true, "500 Internal Server Error");
    let (client, server) = serve_raw_once("GET", PDF_PATH, wire).await;
    let error = client
        .get_installment_payment_book_pdf("ins_123")
        .await
        .unwrap_err();
    server.await.unwrap();
    assert!(matches!(error, Error::RequestFailed { body, .. } if body.len() < 1024));
}

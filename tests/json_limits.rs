#[allow(dead_code)]
mod common;

use asaas::{BillingType, Error, LifecycleError, PaymentUpdateRequest};
use common::serve_raw_once;
const LIMIT: usize = 10 * 1024 * 1024;
const DIAGNOSTIC: &str = "JSON response exceeds the 10 MiB size limit";

fn wire(length: usize, mode: &str, status: &str) -> Vec<u8> {
    let framing = match mode {
        "declared" => format!("Content-Length: {length}\r\n"),
        "chunked" => "Transfer-Encoding: chunked\r\n".to_owned(),
        "closed" => String::new(),
        _ => unreachable!(),
    };
    let mut wire = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{framing}Retry-After: 12\r\nRateLimit-Reset: 30\r\nConnection: close\r\n\r\n").into_bytes();
    // A complete, valid DTO plus whitespace: old unbounded readers accept it.
    let mut body = br#"{"object":"payment","id":"pay_123","status":"PENDING","customer":"cus_123","value":1,"billingType":"BOLETO","dueDate":"2026-12-01","deleted":false}"#.to_vec();
    if status.starts_with("429") {
        body = br#"{"errors":[{"code":"rate_limit","description":"Try later"}]}"#.to_vec();
    }
    body.resize(length, b' ');
    if mode == "chunked" {
        for chunk in body.chunks(64 * 1024) {
            wire.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
            wire.extend_from_slice(chunk);
            wire.extend_from_slice(b"\r\n");
        }
        wire.extend_from_slice(b"0\r\n\r\n");
    } else {
        wire.extend_from_slice(&body);
    }
    wire
}

#[tokio::test]
async fn ordinary_and_lifecycle_accept_exact_limit_and_reject_actual_overflow() {
    for mode in ["declared", "chunked", "closed"] {
        for length in [LIMIT, LIMIT + 1] {
            let (client, server) = serve_raw_once(
                "GET",
                "/v3/payments/pay_123/status",
                wire(length, mode, "200 OK"),
            )
            .await;
            let result = client.get_payment_status("pay_123").await;
            server.await.unwrap();
            if length == LIMIT {
                assert_eq!(result.unwrap().status, asaas::PaymentStatus::Pending);
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.status(), Some(reqwest::StatusCode::OK));
                assert_eq!(error.retry_after(), Some("12"));
                assert!(matches!(error, Error::RequestFailed { body, .. } if body == DIAGNOSTIC));
            }
            let (client, server) =
                serve_raw_once("GET", "/v3/payments/pay_123", wire(length, mode, "200 OK")).await;
            let result = client.get_lifecycle_payment("pay_123").await;
            server.await.unwrap();
            if length == LIMIT {
                assert_eq!(result.unwrap().id, "pay_123");
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.status(), Some(reqwest::StatusCode::OK));
                assert!(
                    matches!(error, LifecycleError::Response { body, decode_error: Some(error), .. } if body == DIAGNOSTIC && error.is_io())
                );
            }
        }
    }
}

#[tokio::test]
async fn oversized_error_keeps_real_status_and_rate_limit_metadata() {
    for mode in ["declared", "chunked", "closed"] {
        let (client, server) = serve_raw_once(
            "GET",
            "/v3/payments/pay_123/status",
            wire(LIMIT + 1, mode, "429 Too Many Requests"),
        )
        .await;
        let error = client.get_payment_status("pay_123").await.unwrap_err();
        server.await.unwrap();
        assert_eq!(error.status(), Some(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert_eq!(error.retry_after(), Some("12"));
        assert_eq!(error.rate_limit_reset_seconds(), Some(30));
        assert!(error.api_errors().is_none());
        assert!(matches!(error, Error::RequestFailed { body, .. } if body == DIAGNOSTIC));
        let (client, server) = serve_raw_once(
            "DELETE",
            "/v3/payments/pay_123",
            wire(LIMIT + 1, mode, "429 Too Many Requests"),
        )
        .await;
        let error = client
            .cancel_lifecycle_payment("pay_123")
            .await
            .unwrap_err();
        server.await.unwrap();
        assert_eq!(error.status(), Some(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert_eq!(error.retry_after(), Some("12"));
        assert_eq!(error.rate_limit_reset_seconds(), Some(30));
        assert!(error.api_errors().is_none());
        assert!(
            matches!(error, LifecycleError::Response { body, decode_error: None, .. } if body == DIAGNOSTIC)
        );
    }
}

#[tokio::test]
async fn declared_overflow_rejects_headers_without_waiting_for_body() {
    for lifecycle in [false, true] {
        let headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            LIMIT + 1
        )
        .into_bytes();
        let target = if lifecycle {
            "/v3/payments/pay_123"
        } else {
            "/v3/payments/pay_123/status"
        };
        let (client, server) = serve_raw_once("GET", target, headers).await;
        if lifecycle {
            assert!(
                matches!(client.get_lifecycle_payment("pay_123").await, Err(LifecycleError::Response { body, decode_error: Some(error), .. }) if body == DIAGNOSTIC && error.is_io())
            );
        } else {
            assert!(
                matches!(client.get_payment_status("pay_123").await, Err(Error::RequestFailed { body, .. }) if body == DIAGNOSTIC)
            );
        }
        server.await.unwrap();
    }
}

#[tokio::test]
async fn successful_mutation_overflow_remains_an_uncertain_received_response_failure() {
    let (client, server) = serve_raw_once(
        "PUT",
        "/v3/payments/pay_123",
        wire(LIMIT + 1, "chunked", "200 OK"),
    )
    .await;
    let payload = PaymentUpdateRequest {
        billing_type: BillingType::Boleto,
        value: 123.45,
        due_date: "2026-12-01".into(),
        description: None,
        days_after_due_date_to_registration_cancellation: None,
        external_reference: None,
        discount: None,
        interest: None,
        fine: None,
        postal_service: None,
        split: None,
        callback: None,
    };
    let error = client
        .update_payment("pay_123", &payload)
        .await
        .unwrap_err();
    server.await.unwrap();
    assert_eq!(error.status(), Some(reqwest::StatusCode::OK));
    assert!(matches!(error, Error::RequestFailed { body, .. } if body == DIAGNOSTIC));

    let (client, server) = serve_raw_once(
        "DELETE",
        "/v3/payments/pay_123",
        wire(LIMIT + 1, "chunked", "200 OK"),
    )
    .await;
    let error = client
        .cancel_lifecycle_payment("pay_123")
        .await
        .unwrap_err();
    server.await.unwrap();
    assert_eq!(error.status(), Some(reqwest::StatusCode::OK));
    assert!(
        matches!(error, LifecycleError::Response { body, decode_error: Some(error), .. } if body == DIAGNOSTIC && error.is_io())
    );
}

#[tokio::test]
async fn mutation_body_read_failure_retains_received_status_and_headers() {
    let broken = b"HTTP/1.1 200 OK\r\nContent-Length: 99\r\nRetry-After: 12\r\nRateLimit-Reset: 30\r\nConnection: close\r\n\r\n{";
    let (client, server) = serve_raw_once("DELETE", "/v3/payments/pay_123", broken.to_vec()).await;
    let error = client
        .cancel_lifecycle_payment("pay_123")
        .await
        .unwrap_err();
    server.await.unwrap();
    assert_eq!(error.status(), Some(reqwest::StatusCode::OK));
    assert_eq!(error.retry_after(), Some("12"));
    assert_eq!(error.rate_limit_reset_seconds(), Some(30));
    assert!(
        matches!(error, LifecycleError::Response { decode_error: Some(error), .. } if error.is_io())
    );

    let (client, server) = serve_raw_once("PUT", "/v3/payments/pay_123", broken.to_vec()).await;
    let payload = PaymentUpdateRequest {
        billing_type: BillingType::Boleto,
        value: 1.0,
        due_date: "2026-12-01".into(),
        description: None,
        days_after_due_date_to_registration_cancellation: None,
        external_reference: None,
        discount: None,
        interest: None,
        fine: None,
        postal_service: None,
        split: None,
        callback: None,
    };
    let error = client
        .update_payment("pay_123", &payload)
        .await
        .unwrap_err();
    server.await.unwrap();
    assert_eq!(error.status(), Some(reqwest::StatusCode::OK));
    assert_eq!(error.retry_after(), Some("12"));
    assert_eq!(error.rate_limit_reset_seconds(), Some(30));
    assert!(matches!(error, Error::RequestFailed { .. }));
}

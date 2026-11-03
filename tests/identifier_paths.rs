mod common;

use asaas::{BillingType, LifecycleInstallmentPaymentListRequest, PaymentUpdateRequest};
use common::{StubResponse, serve_once, serve_raw_once_with_body};

fn update() -> PaymentUpdateRequest {
    PaymentUpdateRequest {
        billing_type: BillingType::Boleto,
        value: 123.45,
        due_date: "2026-12-01".into(),
        description: Some("opaque target".into()),
        days_after_due_date_to_registration_cancellation: None,
        external_reference: None,
        discount: None,
        interest: None,
        fine: None,
        postal_service: None,
        split: None,
        callback: None,
    }
}

#[tokio::test]
async fn mutation_keeps_opaque_identifier_in_one_segment_and_preserves_payload() {
    let payload = update();
    let (client, server) = serve_raw_once_with_body(
        "PUT",
        "/v3/payments/..%2Fcustomers%2Fcus_other%5C%3Fq%3D1%23%C3%A9%25",
        b"HTTP/1.1 400 Bad Request\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_vec(),
        Some(serde_json::to_value(&payload).unwrap()),
    )
    .await;
    let result = client
        .update_payment("../customers/cus_other\\?q=1#é%", &payload)
        .await;
    server.await.unwrap();
    assert_eq!(
        result.unwrap_err().status(),
        Some(reqwest::StatusCode::BAD_REQUEST)
    );
}

#[tokio::test]
async fn every_resource_path_preserves_opaque_identifiers() {
    macro_rules! check {
        ($method:literal, $path:expr, $call:ident $(, $arg:expr)*) => {{
            let mut response = StubResponse::json("{}");
            response.status = "400 Bad Request";
            let (client, server) = serve_once($method, $path, response).await;
            let result = client.$call("a/..\\?x#é%", $($arg),*).await;
            server.await.unwrap();
            assert_eq!(result.unwrap_err().status(), Some(reqwest::StatusCode::BAD_REQUEST));
        }};
    }
    check!(
        "DELETE",
        "/v3/lean/payments/a%2F..%5C%3Fx%23%C3%A9%25",
        delete_payment
    );
    check!(
        "GET",
        "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25/pixQrCode",
        get_payment_pix_qr_code
    );
    check!(
        "GET",
        "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25/identificationField",
        get_payment_identification_field
    );
    check!(
        "GET",
        "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25/status",
        get_payment_status
    );
    check!(
        "GET",
        "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25/billingInfo",
        get_payment_billing_info
    );
    check!("GET", "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25", get_payment);
    check!(
        "GET",
        "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25",
        get_payment_details
    );
    check!(
        "GET",
        "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25",
        get_lifecycle_payment
    );
    check!(
        "DELETE",
        "/v3/payments/a%2F..%5C%3Fx%23%C3%A9%25",
        cancel_lifecycle_payment
    );
    check!(
        "GET",
        "/v3/customers/a%2F..%5C%3Fx%23%C3%A9%25",
        get_customer
    );
    check!(
        "GET",
        "/v3/installments/a%2F..%5C%3Fx%23%C3%A9%25/payments?offset=7",
        list_installment_payments,
        Some(7)
    );
    check!(
        "GET",
        "/v3/installments/a%2F..%5C%3Fx%23%C3%A9%25/paymentBook",
        get_installment_payment_book_pdf
    );
    check!(
        "GET",
        "/v3/installments/a%2F..%5C%3Fx%23%C3%A9%25",
        get_lifecycle_installment
    );
    check!(
        "GET",
        "/v3/installments/a%2F..%5C%3Fx%23%C3%A9%25/payments?offset=7&limit=2",
        list_lifecycle_installment_payments,
        &LifecycleInstallmentPaymentListRequest {
            offset: Some(7),
            limit: Some(2)
        }
    );
}

#[tokio::test]
async fn encoded_looking_and_non_segment_dots_remain_opaque() {
    for (id, path) in [
        ("%2e", "/v3/payments/%252e/status"),
        ("%2e%2e", "/v3/payments/%252e%252e/status"),
        ("...", "/v3/payments/.../status"),
        ("pay_123.a-~", "/v3/payments/pay_123.a-~/status"),
    ] {
        let (client, server) =
            serve_once("GET", path, StubResponse::json(r#"{"status":"PENDING"}"#)).await;
        client.get_payment_status(id).await.unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn empty_and_dot_segments_are_safe_local_errors_before_any_io() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let http = reqwest::Client::builder()
        .no_proxy()
        .resolve("api-sandbox.asaas.com", listener.local_addr().unwrap())
        .timeout(std::time::Duration::from_millis(50))
        .build()
        .unwrap();
    let client = asaas::ClientBuilder::new()
        .api_key("secret")
        .user_agent("test")
        .http_client(http)
        .build()
        .unwrap();
    let payload = update();
    for id in ["", ".", ".."] {
        macro_rules! check_error {
            ($variant:path, $call:ident $(, $arg:expr)*) => {{
                let error = client.$call(id, $($arg),*).await.unwrap_err();
                assert!(matches!(error, $variant));
                assert_eq!(format!("{error:?}"), "InvalidIdentifier");
                assert_eq!(error.to_string(), "invalid resource identifier");
                assert!(error.status().is_none());
                assert!(error.retry_after().is_none());
                assert!(error.rate_limit_reset_seconds().is_none());
                assert!(error.api_errors().is_none());
                assert!(std::error::Error::source(&error).is_none());
            }};
        }
        check_error!(asaas::Error::InvalidIdentifier, update_payment, &payload);
        check_error!(asaas::Error::InvalidIdentifier, delete_payment);
        check_error!(asaas::Error::InvalidIdentifier, get_payment_pix_qr_code);
        check_error!(
            asaas::Error::InvalidIdentifier,
            get_payment_identification_field
        );
        check_error!(asaas::Error::InvalidIdentifier, get_payment_status);
        check_error!(asaas::Error::InvalidIdentifier, get_payment_billing_info);
        check_error!(asaas::Error::InvalidIdentifier, get_payment);
        check_error!(asaas::Error::InvalidIdentifier, get_payment_details);
        check_error!(
            asaas::Error::InvalidIdentifier,
            list_installment_payments,
            None
        );
        check_error!(
            asaas::Error::InvalidIdentifier,
            get_installment_payment_book_pdf
        );
        check_error!(
            asaas::LifecycleError::InvalidIdentifier,
            get_lifecycle_payment
        );
        check_error!(
            asaas::LifecycleError::InvalidIdentifier,
            cancel_lifecycle_payment
        );
        check_error!(asaas::LifecycleError::InvalidIdentifier, get_customer);
        check_error!(
            asaas::LifecycleError::InvalidIdentifier,
            get_lifecycle_installment
        );
        check_error!(
            asaas::LifecycleError::InvalidIdentifier,
            list_lifecycle_installment_payments,
            &LifecycleInstallmentPaymentListRequest::default()
        );
    }
    assert!(
        client
            .get_customer(".")
            .await
            .unwrap_err()
            .response_body()
            .is_none()
    );
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(50), listener.accept())
            .await
            .is_err(),
        "local rejection must not open a connection"
    );
}

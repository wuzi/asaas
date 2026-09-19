//! Fixtures follow Asaas' public v3 schemas, reviewed 2026-09-18.
//! https://docs.asaas.com/reference/listar-cobrancas
//! https://docs.asaas.com/reference/listar-clientes
//! https://docs.asaas.com/reference/retrieve-a-single-installment
//! https://docs.asaas.com/reference/delete-payment
mod common;

use asaas::{
    CustomerListRequest, LifecycleInstallmentPaymentListRequest, LifecyclePaymentListRequest,
};
use common::{StubResponse, serve_once};

const PAYMENT: &str = r#"{
  "object":"payment",
  "id":"pay_123",
  "customer":"cus_123",
  "externalReference":"invoice/42 + one",
  "status":"FUTURE_STATUS",
  "billingType":"FUTURE_TYPE",
  "value":9007199254740993.01,
  "originalValue":129.900000000000000001,
  "dueDate":"2026-10-01",
  "deleted":false,
  "installment":"ins_123",
  "installmentNumber":2,
  "split":[{
    "id":"split_123",
    "walletId":"wallet_123",
    "fixedValue":10.000000000000000001,
    "percentualValue":14.285714285714285714,
    "totalValue":20.320000000000000001,
    "status":"PENDING",
    "externalReference":"recipient/1",
    "description":"Lessor split"
  }]
}"#;

fn payment_page() -> String {
    format!(
        r#"{{"object":"list","hasMore":true,"totalCount":2,"limit":1,"offset":0,"data":[{PAYMENT}]}}"#
    )
}

#[tokio::test]
async fn payment_listing_encodes_lifecycle_filters_and_preserves_exact_observation() {
    let body = Box::leak(payment_page().into_boxed_str());
    let (client, server) = serve_once(
        "GET",
        "/v3/payments?offset=0&limit=1&customer=cus_123&externalReference=invoice%2F42%20%2B%20one",
        StubResponse::json(body),
    )
    .await;

    let response = client
        .list_lifecycle_payments(&LifecyclePaymentListRequest {
            offset: Some(0),
            limit: Some(1),
            customer: Some("cus_123".into()),
            external_reference: Some("invoice/42 + one".into()),
        })
        .await
        .unwrap();

    assert!(response.has_more);
    assert_eq!(response.total_count, 2);
    assert_eq!(response.limit, 1);
    assert_eq!(response.offset, 0);
    let payment = &response.data[0];
    assert_eq!(payment.id, "pay_123");
    assert_eq!(payment.customer, "cus_123");
    assert_eq!(
        payment.external_reference.as_deref(),
        Some("invoice/42 + one")
    );
    assert_eq!(payment.status, "FUTURE_STATUS");
    assert_eq!(payment.billing_type, "FUTURE_TYPE");
    assert_eq!(payment.value.to_string(), "9007199254740993.01");
    assert_eq!(
        payment.original_value.as_ref().unwrap().to_string(),
        "129.900000000000000001"
    );
    assert_eq!(payment.due_date, "2026-10-01");
    assert_eq!(payment.deleted, Some(false));
    assert_eq!(payment.installment.as_deref(), Some("ins_123"));
    assert_eq!(payment.installment_number, Some(2));
    let split = &payment.split[0];
    assert_eq!(split.id.as_deref(), Some("split_123"));
    assert_eq!(split.wallet_id.as_deref(), Some("wallet_123"));
    assert_eq!(
        split.fixed_value.as_ref().unwrap().to_string(),
        "10.000000000000000001"
    );
    assert_eq!(
        split.percentual_value.as_ref().unwrap().to_string(),
        "14.285714285714285714"
    );
    assert_eq!(
        split.total_value.as_ref().unwrap().to_string(),
        "20.320000000000000001"
    );
    server.await.unwrap();
}

#[tokio::test]
async fn direct_payment_read_encodes_identity_and_returns_lifecycle_shape() {
    let (client, server) =
        serve_once("GET", "/v3/payments/pay%2F123", StubResponse::json(PAYMENT)).await;

    let payment = client.get_lifecycle_payment("pay/123").await.unwrap();

    assert_eq!(payment.id, "pay_123");
    assert_eq!(payment.customer, "cus_123");
    server.await.unwrap();
}

#[tokio::test]
async fn customer_listing_encodes_reference_and_tax_id_filters() {
    let (client, server) = serve_once(
        "GET",
        "/v3/customers?offset=20&limit=10&cpfCnpj=12345678901&externalReference=tenant%2F42",
        StubResponse::json(
            r#"{"object":"list","hasMore":false,"totalCount":1,"limit":10,"offset":20,"data":[{"object":"customer","id":"cus_123","name":"Ada","cpfCnpj":"12345678901","externalReference":"tenant/42","deleted":false}]}"#,
        ),
    )
    .await;

    let response = client
        .list_customers(&CustomerListRequest {
            offset: Some(20),
            limit: Some(10),
            cpf_cnpj: Some("12345678901".into()),
            external_reference: Some("tenant/42".into()),
        })
        .await
        .unwrap();

    assert!(!response.has_more);
    assert_eq!(response.data[0].id, "cus_123");
    assert_eq!(response.data[0].cpf_cnpj.as_deref(), Some("12345678901"));
    assert_eq!(
        response.data[0].external_reference.as_deref(),
        Some("tenant/42")
    );
    server.await.unwrap();
}

#[tokio::test]
async fn direct_customer_read_encodes_identity() {
    let (client, server) = serve_once(
        "GET",
        "/v3/customers/cus%2F123",
        StubResponse::json(
            r#"{"object":"customer","id":"cus_123","cpfCnpj":"12345678901","externalReference":"tenant/42"}"#,
        ),
    )
    .await;

    let customer = client.get_customer("cus/123").await.unwrap();

    assert_eq!(customer.id, "cus_123");
    assert_eq!(customer.external_reference.as_deref(), Some("tenant/42"));
    server.await.unwrap();
}

#[tokio::test]
async fn installment_group_read_preserves_exact_values() {
    let (client, server) = serve_once(
        "GET",
        "/v3/installments/ins%2F123",
        StubResponse::json(
            r#"{"object":"installment","id":"ins_123","value":360.000000000000000001,"netValue":312.120000000000000001,"paymentValue":30.000000000000000001,"installmentCount":12,"billingType":"FUTURE_TYPE","customer":"cus_123","deleted":false}"#,
        ),
    )
    .await;

    let installment = client.get_lifecycle_installment("ins/123").await.unwrap();

    assert_eq!(installment.id, "ins_123");
    assert_eq!(
        installment.value.as_ref().unwrap().to_string(),
        "360.000000000000000001"
    );
    assert_eq!(
        installment.payment_value.as_ref().unwrap().to_string(),
        "30.000000000000000001"
    );
    assert_eq!(installment.installment_count, Some(12));
    assert_eq!(installment.billing_type.as_deref(), Some("FUTURE_TYPE"));
    assert_eq!(installment.customer.as_deref(), Some("cus_123"));
    server.await.unwrap();
}

#[tokio::test]
async fn installment_children_read_is_paginated_and_exact() {
    let body = Box::leak(payment_page().into_boxed_str());
    let (client, server) = serve_once(
        "GET",
        "/v3/installments/ins%2F123/payments?offset=1&limit=100",
        StubResponse::json(body),
    )
    .await;

    let response = client
        .list_lifecycle_installment_payments(
            "ins/123",
            &LifecycleInstallmentPaymentListRequest {
                offset: Some(1),
                limit: Some(100),
            },
        )
        .await
        .unwrap();

    assert_eq!(response.data[0].value.to_string(), "9007199254740993.01");
    assert_eq!(response.data[0].installment.as_deref(), Some("ins_123"));
    server.await.unwrap();
}

#[tokio::test]
async fn cancellation_uses_standard_endpoint_and_requires_positive_identity() {
    let (client, server) = serve_once(
        "DELETE",
        "/v3/payments/pay%2F123",
        StubResponse::json(r#"{"deleted":true,"id":"pay_123"}"#),
    )
    .await;

    let cancellation = client.cancel_lifecycle_payment("pay/123").await.unwrap();

    assert!(cancellation.deleted);
    assert_eq!(cancellation.id, "pay_123");
    server.await.unwrap();
}

#[tokio::test]
async fn lifecycle_error_retains_status_body_structured_errors_and_retry_metadata() {
    let body = r#"{"errors":[{"code":"rate_limit","description":"Try later"}]}"#;
    let (client, server) = serve_once(
        "GET",
        "/v3/payments/pay_123",
        StubResponse {
            status: "429 Too Many Requests",
            headers: &[
                ("Retry-After", "Wed, 21 Oct 2015 07:28:00 GMT"),
                ("RateLimit-Reset", "17"),
            ],
            body,
        },
    )
    .await;

    let error = client.get_lifecycle_payment("pay_123").await.unwrap_err();

    assert_eq!(error.status(), Some(reqwest::StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(error.retry_after(), Some("Wed, 21 Oct 2015 07:28:00 GMT"));
    assert_eq!(error.response_body(), Some(body));
    assert_eq!(error.rate_limit_reset_seconds(), Some(17));
    let errors = error.api_errors().unwrap();
    assert_eq!(errors[0].code, "rate_limit");
    assert_eq!(errors[0].description, "Try later");
    server.await.unwrap();
}

#[tokio::test]
async fn lifecycle_decode_error_retains_success_status_and_raw_body() {
    let (client, server) = serve_once(
        "GET",
        "/v3/payments/pay_123",
        StubResponse::json("not-json"),
    )
    .await;

    let error = client.get_lifecycle_payment("pay_123").await.unwrap_err();

    assert_eq!(error.status(), Some(reqwest::StatusCode::OK));
    assert_eq!(error.response_body(), Some("not-json"));
    assert!(std::error::Error::source(&error).is_some());
    server.await.unwrap();
}

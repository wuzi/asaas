//! Fixtures follow the public full-payment and wallet schemas, fetched 2026-09-18.
//! https://docs.asaas.com/reference/recuperar-uma-unica-cobranca
//! https://docs.asaas.com/reference/recuperar-walletid
mod common;

use common::{StubResponse, serve_once};

#[tokio::test]
async fn full_payment_read_preserves_exact_numbers_and_unknown_native_values() {
    let (client, server) = serve_once("GET", "/v3/payments/pay_123", StubResponse::json(r#"{"id":"pay_123","status":"FUTURE_STATUS","billingType":"FUTURE_TYPE","value":9007199254740993.01,"originalValue":129.900000000000000001,"paymentDate":"2026-09-17","installment":"ins_123","installmentNumber":2}"#)).await;
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
    let (client, server) = serve_once("GET", "/v3/wallets/", StubResponse::json(r#"{"object":"list","hasMore":false,"totalCount":1,"limit":10,"offset":0,"data":[{"object":"wallet","id":"wallet-123"}]}"#)).await;
    let wallets = client.get_wallets().await.unwrap();
    assert_eq!(wallets.data[0].id, "wallet-123");
    assert!(!wallets.has_more);
    server.await.unwrap();
}

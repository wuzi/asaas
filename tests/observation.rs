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
async fn payment_instructions_keep_native_id_in_one_path_segment() {
    let (client, server) = serve_once(
        "GET",
        "/v3/payments/pay%2F123/billingInfo",
        StubResponse::json("{}"),
    )
    .await;
    client.get_payment_billing_info("pay/123").await.unwrap();
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

#[tokio::test]
async fn full_payment_details_preserve_exact_refund_evidence() {
    let (client, server) = serve_once("GET", "/v3/payments/pay%2F123", StubResponse::json(r#"{"id":"pay/123","externalReference":"hubnari:operation:fixture","status":"RECEIVED","billingType":"BOLETO","value":100.00,"refunds":[{"status":"DONE","value":1.000000000000000001,"refundedSplits":[{"id":"split_1","value":0.50,"done":true}]}]}"#)).await;
    let details = client.get_payment_details("pay/123").await.unwrap();
    assert_eq!(details.payment.id, "pay/123");
    assert_eq!(
        details.external_reference.as_deref(),
        Some("hubnari:operation:fixture")
    );
    assert_eq!(details.refunds[0]["status"], "DONE");
    assert_eq!(
        details.refunds[0]["value"].as_number().unwrap().to_string(),
        "1.000000000000000001"
    );
    assert_eq!(details.refunds[0]["refundedSplits"][0]["done"], true);
    server.await.unwrap();
}

#[test]
fn payment_details_distinguish_absent_refunds_from_ambiguous_provider_data() {
    let base = serde_json::json!({"id":"pay_123","status":"RECEIVED","billingType":"BOLETO","value":100.00});
    let absent: asaas::PaymentDetails = serde_json::from_value(base.clone()).unwrap();
    assert_eq!(absent.refunds, serde_json::json!([]));
    for deleted in [serde_json::Value::Null, false.into(), true.into()] {
        let mut native = base.clone();
        native["deleted"] = deleted.clone();
        let details: asaas::PaymentDetails = serde_json::from_value(native).unwrap();
        assert_eq!(
            serde_json::to_value(details).unwrap().get("deleted"),
            Some(&deleted),
            "native deletion evidence must survive the authenticated read"
        );
    }
    for refunds in [
        serde_json::Value::Null,
        serde_json::json!({"status":"UNKNOWN"}),
        serde_json::json!([{"status":"FUTURE", "value":"invalid"}]),
    ] {
        let mut native = base.clone();
        native["refunds"] = refunds.clone();
        let details: asaas::PaymentDetails = serde_json::from_value(native).unwrap();
        assert_eq!(details.refunds, refunds);
        let serialized = serde_json::to_value(details).unwrap();
        assert_eq!(
            serialized["refunds"], refunds,
            "proof must retain raw ambiguous refund data"
        );
    }
}

use asaas::{LifecyclePaymentDiscountResponse, LifecyclePaymentResponse};

#[test]
fn fixed_discount_fixture_preserves_exact_value_and_deadline_fields() {
    let payment: LifecyclePaymentResponse = serde_json::from_str(include_str!(
        "fixtures/lifecycle-payment-discount-fixed.json"
    ))
    .unwrap();

    let discount = payment.discount.unwrap();
    assert_eq!(
        discount.value.to_string(),
        "9007199254740993.000000000000000001"
    );
    assert_eq!(discount.kind, "FIXED");
    assert_eq!(discount.due_date_limit_days, Some(5));
    assert_eq!(discount.limit_date.as_deref(), Some("2026-10-26"));
}

#[test]
fn percentage_discount_fixture_preserves_exact_value_and_null_deadlines() {
    let payment: LifecyclePaymentResponse = serde_json::from_str(include_str!(
        "fixtures/lifecycle-payment-discount-percentage.json"
    ))
    .unwrap();

    let discount = payment.discount.unwrap();
    assert_eq!(discount.value.to_string(), "14.285714285714285714");
    assert_eq!(discount.kind, "PERCENTAGE");
    assert_eq!(discount.due_date_limit_days, None);
    assert_eq!(discount.limit_date, None);
}

#[test]
fn absent_and_null_discount_preserve_existing_lifecycle_responses() {
    for body in [
        r#"{"id":"pay_123","customer":"cus_123","status":"PENDING","billingType":"BOLETO","value":129.9,"dueDate":"2026-10-31"}"#,
        r#"{"id":"pay_123","customer":"cus_123","status":"PENDING","billingType":"BOLETO","value":129.9,"dueDate":"2026-10-31","discount":null}"#,
    ] {
        let payment: LifecyclePaymentResponse = serde_json::from_str(body).unwrap();
        assert!(payment.discount.is_none());
        assert!(payment.split.is_empty());
    }
}

#[test]
fn discount_response_roundtrip_keeps_exact_numbers_and_native_field_names() {
    let body = r#"{"value":14.285714285714285714,"type":"PERCENTAGE","dueDateLimitDays":0,"limitDate":"2026-10-31"}"#;
    let discount: LifecyclePaymentDiscountResponse = serde_json::from_str(body).unwrap();

    let serialized = serde_json::to_string(&discount).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    let expected: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(actual, expected);

    let reparsed: LifecyclePaymentDiscountResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(reparsed.value.to_string(), "14.285714285714285714");
    assert_eq!(reparsed.kind, "PERCENTAGE");
    assert_eq!(reparsed.due_date_limit_days, Some(0));
    assert_eq!(reparsed.limit_date.as_deref(), Some("2026-10-31"));
}

#[test]
fn discount_response_accepts_future_native_kind_and_absent_deadlines() {
    let discount: LifecyclePaymentDiscountResponse =
        serde_json::from_str(r#"{"value":0,"type":"FUTURE_TYPE"}"#).unwrap();

    assert_eq!(discount.value.to_string(), "0");
    assert_eq!(discount.kind, "FUTURE_TYPE");
    assert_eq!(discount.due_date_limit_days, None);
    assert_eq!(discount.limit_date, None);
}

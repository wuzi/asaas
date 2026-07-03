use asaas::{
    BillingType, ClientBuilder, Environment, PaymentCallback, PaymentDiscount, PaymentFine,
    PaymentInterest, PaymentSplitItem, PaymentUpdateRequest,
};
use serde_json::json;

fn payment_update_request() -> PaymentUpdateRequest {
    PaymentUpdateRequest {
        billing_type: BillingType::Boleto,
        value: 129.9,
        due_date: "2017-06-10".into(),
        description: Some("Pedido 056984".into()),
        days_after_due_date_to_registration_cancellation: Some(1),
        external_reference: Some("056984".into()),
        discount: Some(PaymentDiscount {
            value: 10.0,
            limit_date: None,
            due_date_limit_days: Some(0),
            kind: Some("PERCENTAGE".into()),
        }),
        interest: Some(PaymentInterest { value: Some(2.0) }),
        fine: Some(PaymentFine {
            value: Some(1.0),
            kind: Some("FIXED".into()),
        }),
        postal_service: Some(false),
        split: Some(vec![PaymentSplitItem {
            wallet_id: "wal_123".into(),
            fixed_value: Some(10.0),
            percentual_value: None,
            total_fixed_value: None,
            external_reference: Some("split-1".into()),
            description: Some("Seller split".into()),
        }]),
        callback: Some(PaymentCallback {
            success_url: "https://example.com/success".into(),
            auto_redirect: Some(true),
        }),
    }
}

#[test]
fn update_payment_request_serializes_with_asaas_field_names() {
    let payload = payment_update_request();

    let serialized = serde_json::to_value(&payload).unwrap();

    assert_eq!(
        serialized,
        json!({
            "billingType": "BOLETO",
            "value": 129.9,
            "dueDate": "2017-06-10",
            "description": "Pedido 056984",
            "daysAfterDueDateToRegistrationCancellation": 1,
            "externalReference": "056984",
            "discount": {
                "value": 10.0,
                "dueDateLimitDays": 0,
                "type": "PERCENTAGE"
            },
            "interest": {
                "value": 2.0
            },
            "fine": {
                "value": 1.0,
                "type": "FIXED"
            },
            "postalService": false,
            "split": [{
                "walletId": "wal_123",
                "fixedValue": 10.0,
                "externalReference": "split-1",
                "description": "Seller split"
            }],
            "callback": {
                "successUrl": "https://example.com/success",
                "autoRedirect": true
            }
        })
    );
    assert!(serialized.get("customer").is_none());
}

#[test]
fn update_payment_method_is_available_on_client() {
    let client = ClientBuilder::new()
        .api_key("test-key")
        .user_agent("asaas-tests")
        .environment(Environment::Sandbox)
        .build()
        .unwrap();
    let payload = payment_update_request();

    let future = client.update_payment("pay_080225913252", &payload);
    drop(future);
}

# asaas

A Rust SDK for integrating with the payment services provided by [Asaas](https://docs.asaas.com/). This library provides strongly-typed bindings for the Asaas API.

# Quick Start

```rs
use asaas::{
    BillingType, ClientBuilder, Environment, LeanPaymentCreateRequest, PaymentDiscount,
    PaymentFine, PaymentInterest,
};

let client = ClientBuilder::new()
    .api_key("your_api_key")
    .user_agent("application name")
    .environment(Environment::Sandbox)
    .build()?;

let payload = LeanPaymentCreateRequest {
    customer: "cus_G7Dvo4iphUNk".into(),
    billing_type: BillingType::Boleto,
    value: 129.9,
    due_date: "2017-06-10".into(),
    description: Some("Pedido 056984".into()),
    days_after_due_date_to_registration_cancellation: Some(1),
    external_reference: Some("056984".into()),
    installment_count: None,
    total_value: None,
    installment_value: None,
    discount: Some(PaymentDiscount {
        value: 10.0,
        due_date_limit_days: Some(0),
        kind: Some("PERCENTAGE".into()),
    }),
    interest: Some(PaymentInterest { value: None }),
    fine: Some(PaymentFine {
        value: None,
        kind: Some("FIXED".into()),
    }),
    postal_service: Some(false),
    split: None,
    callback: None,
    pix_automatic_authorization_id: Some("89060430-aceb-447c-a981-07ee15daf00c".into()),
};

let response = client.create_lean_payment(&payload).await?;
```

# Implemented Endpoints

- `POST /v3/customers` -> `client.create_customer(...)`
- `POST /v3/lean/payments` -> `client.create_lean_payment(...)`
- `PUT /v3/payments/{id}` -> `client.update_payment(...)`
- `GET /v3/payments/{id}/pixQrCode` -> `client.get_payment_pix_qr_code(...)`
- `GET /v3/payments/{id}/identificationField` -> `client.get_payment_identification_field(...)`
- `GET /v3/payments/{id}/billingInfo` -> `client.get_payment_billing_info(...)`
- `GET /v3/installments/{id}/payments?offset={offset}` -> `client.list_installment_payments(...)`
- `GET /v3/installments/{id}/paymentBook` -> `client.get_installment_payment_book_pdf(...)`


### Read-only reconciliation

`Client::get_payment(id)` uses the authenticated public `GET /v3/payments/{id}`.
`PaymentResponse` preserves monetary fields as `serde_json::Number` with arbitrary
precision; parse their decimal text directly into your decimal library. Status and
billing type remain native strings so new provider values deserialize safely.
`Client::get_wallets()` uses `GET /v3/wallets/` to identify the authenticated account.
Consumers must reject ambiguous wallet lists and verify requested payment IDs.

The loopback TLS tests in `tests/observation.rs` exercise these documented schemas:
https://docs.asaas.com/reference/recuperar-uma-unica-cobranca and
https://docs.asaas.com/reference/recuperar-walletid (reviewed 2026-09-18).
Fixtures use synthetic identifiers and no provider credentials or external calls.

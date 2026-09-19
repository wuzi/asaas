# asaas

A Rust SDK for integrating with the payment services provided by [Asaas](https://docs.asaas.com/). This library provides strongly-typed bindings for the Asaas API.

Version 0.2 adds deletion evidence to `PaymentDetails` and makes that response type
non-exhaustive. Obtain it through `get_payment_details` or deserialize a response
instead of constructing a struct literal. An absent `deleted` value is distinct
from `Some(false)`; refund records continue to retain exact JSON numbers.

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

`Client::get_payment_details(id)` uses the authenticated public `GET /v3/payments/{id}`
and preserves every `refunds` record alongside the payment. Missing refunds become
an empty array; explicit null, unknown fields, and malformed records are preserved
for the caller to validate. A paid status alone does not prove retained funds;
inspect refund attempts before settling. The existing `get_payment` summary and
`PaymentResponse` struct fields remain source compatible.

`PaymentResponse` preserves monetary fields as `serde_json::Number` with arbitrary
precision; parse their decimal text directly into your decimal library. Status and
billing type remain native strings so new provider values deserialize safely.
`Client::get_wallets()` uses `GET /v3/wallets/` to identify the authenticated account.
Consumers must reject ambiguous wallet lists and verify requested payment IDs.

The loopback TLS tests in `tests/observation.rs` exercise these documented schemas:
https://docs.asaas.com/reference/recuperar-uma-unica-cobranca and
https://docs.asaas.com/reference/recuperar-walletid and
https://docs.asaas.com/docs/refunds (reviewed 2026-09-18).
Fixtures use synthetic identifiers and no provider credentials or external calls.

## HTTP transport and request accounting

Installment PDF downloads are limited to 10 MiB, including responses without a
Content-Length header and provider error bodies. Successful downloads must have
the PDF media type and PDF header/end markers; valid PDF bytes are returned
unchanged. Oversized or invalid PDF responses use the existing
`Error::RequestFailed` variant with a short diagnostic and the native HTTP status.

SDK-created clients disable automatic HTTP retries and redirects. Each HTTP
attempt has a 20-second total timeout and a 5-second connection timeout, including
TLS negotiation. A timeout or dropped response does not prove a mutation failed;
reconcile uncertain mutations before deciding whether to submit another request.

`ClientBuilder::http_client` accepts a caller-owned Reqwest client. Configure that
client with `.retry(reqwest::retry::never())`,
`.redirect(reqwest::redirect::Policy::none())`, and finite `.timeout(...)` and
`.connect_timeout(...)` values to preserve these accounting bounds.

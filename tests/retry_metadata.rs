mod common;

use asaas::CreateCustomerRequest;
use common::{StubResponse, serve_once};

#[tokio::test]
async fn ordinary_post_exposes_rate_limit_metadata_without_a_retry() {
    let mut rate_limited =
        StubResponse::json(r#"{"errors":[{"code":"rate_limit","description":"Try later"}]}"#);
    rate_limited.status = "429 Too Many Requests";
    rate_limited.headers = &[("Retry-After", "12"), ("RateLimit-Reset", "30")];
    let (client, server) = serve_once("POST", "/v3/customers", rate_limited).await;

    let error = client
        .create_customer(&CreateCustomerRequest {
            name: "Retry metadata".into(),
            cpf_cnpj: "94271564656".into(),
            email: None,
            phone: None,
            mobile_phone: None,
            address: None,
            address_number: None,
            complement: None,
            province: None,
            postal_code: None,
            external_reference: None,
            notification_disabled: None,
            additional_emails: None,
            municipal_inscription: None,
            state_inscription: None,
            observations: None,
            group_name: None,
            company: None,
            foreign_customer: None,
        })
        .await
        .unwrap_err();

    assert_eq!(error.status(), Some(reqwest::StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(error.retry_after(), Some("12"));
    assert_eq!(error.rate_limit_reset_seconds(), Some(30));
    server.await.unwrap();
}

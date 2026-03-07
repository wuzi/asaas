use reqwest::Method;

use crate::client::Client;
use crate::error::Error;
use crate::types::{LeanPaymentCreateRequest, LeanPaymentResponse};

impl Client {
    pub async fn create_lean_payment(
        &self,
        payload: &LeanPaymentCreateRequest,
    ) -> Result<LeanPaymentResponse, Error> {
        self.send_typed(Method::POST, "/v3/lean/payments", Some(payload))
            .await
    }
}

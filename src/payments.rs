use reqwest::Method;

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    BillingInfoResponse, IdentificationFieldResponse, LeanPaymentCreateRequest,
    LeanPaymentDeleteResponse, LeanPaymentResponse, PaymentStatusResponse, PaymentUpdateRequest,
    PixQrCodeResponse,
};

impl Client {
    pub async fn create_lean_payment(
        &self,
        payload: &LeanPaymentCreateRequest,
    ) -> Result<LeanPaymentResponse, Error> {
        self.send_typed(Method::POST, "/v3/lean/payments", Some(payload))
            .await
    }

    pub async fn update_payment(
        &self,
        payment_id: &str,
        payload: &PaymentUpdateRequest,
    ) -> Result<LeanPaymentResponse, Error> {
        let path = format!("/v3/payments/{payment_id}");
        self.send_typed(Method::PUT, &path, Some(payload)).await
    }

    pub async fn delete_payment(
        &self,
        payment_id: &str,
    ) -> Result<LeanPaymentDeleteResponse, Error> {
        let path = format!("/v3/lean/payments/{payment_id}");
        self.send_typed::<LeanPaymentDeleteResponse, _>(Method::DELETE, &path, None)
            .await
    }

    pub async fn get_payment_pix_qr_code(
        &self,
        payment_id: &str,
    ) -> Result<PixQrCodeResponse, Error> {
        let path = format!("/v3/payments/{payment_id}/pixQrCode");
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn get_payment_identification_field(
        &self,
        payment_id: &str,
    ) -> Result<IdentificationFieldResponse, Error> {
        let path = format!("/v3/payments/{payment_id}/identificationField");
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn get_payment_status(
        &self,
        payment_id: &str,
    ) -> Result<PaymentStatusResponse, Error> {
        let path = format!("/v3/payments/{payment_id}/status");
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn get_payment_billing_info(
        &self,
        payment_id: &str,
    ) -> Result<BillingInfoResponse, Error> {
        let path = format!("/v3/payments/{payment_id}/billingInfo");
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }
}

impl Client {
    /// Read all payment fields used for reconciliation with exact decimal tokens.
    pub async fn get_payment(
        &self,
        payment_id: &str,
    ) -> Result<crate::types::PaymentResponse, Error> {
        let encoded: String = payment_id
            .bytes()
            .map(|byte| {
                if byte.is_ascii_alphanumeric() || b"_-".contains(&byte) {
                    char::from(byte).to_string()
                } else {
                    format!("%{byte:02X}")
                }
            })
            .collect();
        self.send_typed::<(), _>(Method::GET, &format!("/v3/payments/{encoded}"), None)
            .await
    }

    /// Identify the authenticated receiving account without private financial APIs.
    pub async fn get_wallets(&self) -> Result<crate::types::WalletsResponse, Error> {
        self.send_typed::<(), _>(Method::GET, "/v3/wallets/", None)
            .await
    }
}

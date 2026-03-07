use reqwest::Method;

use crate::client::Client;
use crate::error::Error;
use crate::types::{
    IdentificationFieldResponse, LeanPaymentCreateRequest, LeanPaymentResponse, PixQrCodeResponse,
};

impl Client {
    pub async fn create_lean_payment(
        &self,
        payload: &LeanPaymentCreateRequest,
    ) -> Result<LeanPaymentResponse, Error> {
        self.send_typed(Method::POST, "/v3/lean/payments", Some(payload))
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
}

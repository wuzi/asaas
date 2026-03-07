use std::fmt::Write;

use reqwest::Method;

use crate::client::Client;
use crate::error::Error;
use crate::types::InstallmentPaymentsListResponse;

impl Client {
    pub async fn list_installment_payments(
        &self,
        installment_id: &str,
        offset: Option<u64>,
    ) -> Result<InstallmentPaymentsListResponse, Error> {
        let mut path = format!("/v3/installments/{installment_id}/payments");
        if let Some(value) = offset {
            let _ = write!(path, "?offset={value}");
        }
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn get_installment_payment_book_pdf(
        &self,
        installment_id: &str,
    ) -> Result<Vec<u8>, Error> {
        let path = format!("/v3/installments/{installment_id}/paymentBook");
        self.send_bytes(Method::GET, &path, "application/pdf").await
    }
}

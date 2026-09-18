use std::fmt::Write;

use reqwest::Method;

use crate::client::{Client, encode_url_component, push_query};
use crate::error::{Error, LifecycleError};
use crate::types::{
    InstallmentPaymentsListResponse, LifecycleInstallmentPaymentListRequest,
    LifecycleInstallmentResponse, LifecyclePaymentListResponse,
};

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

    pub async fn get_lifecycle_installment(
        &self,
        installment_id: &str,
    ) -> Result<LifecycleInstallmentResponse, LifecycleError> {
        let installment_id = encode_url_component(installment_id);
        self.send_lifecycle_typed(Method::GET, &format!("/v3/installments/{installment_id}"))
            .await
    }

    pub async fn list_lifecycle_installment_payments(
        &self,
        installment_id: &str,
        request: &LifecycleInstallmentPaymentListRequest,
    ) -> Result<LifecyclePaymentListResponse, LifecycleError> {
        let installment_id = encode_url_component(installment_id);
        let mut path = format!("/v3/installments/{installment_id}/payments");
        let mut has_query = false;
        if let Some(value) = request.offset {
            push_query(&mut path, &mut has_query, "offset", &value.to_string());
        }
        if let Some(value) = request.limit {
            push_query(&mut path, &mut has_query, "limit", &value.to_string());
        }
        self.send_lifecycle_typed(Method::GET, &path).await
    }
}

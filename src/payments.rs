use reqwest::Method;

use crate::client::Client;
use crate::client::{encode_identifier, push_query};
use crate::error::{Error, LifecycleError};
use crate::types::{
    BillingInfoResponse, IdentificationFieldResponse, LeanPaymentCreateRequest,
    LeanPaymentDeleteResponse, LeanPaymentResponse, LifecyclePaymentCancellationResponse,
    LifecyclePaymentListRequest, LifecyclePaymentListResponse, LifecyclePaymentResponse,
    PaymentStatusResponse, PaymentUpdateRequest, PixQrCodeResponse,
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
        let path = payment_path(payment_id)?;
        self.send_typed(Method::PUT, &path, Some(payload)).await
    }

    pub async fn delete_payment(
        &self,
        payment_id: &str,
    ) -> Result<LeanPaymentDeleteResponse, Error> {
        let payment_id = encode_identifier(payment_id).ok_or(Error::InvalidIdentifier)?;
        let path = format!("/v3/lean/payments/{payment_id}");
        self.send_typed::<LeanPaymentDeleteResponse, _>(Method::DELETE, &path, None)
            .await
    }

    pub async fn get_payment_pix_qr_code(
        &self,
        payment_id: &str,
    ) -> Result<PixQrCodeResponse, Error> {
        let path = format!("{}/pixQrCode", payment_path(payment_id)?);
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn get_payment_identification_field(
        &self,
        payment_id: &str,
    ) -> Result<IdentificationFieldResponse, Error> {
        let path = format!("{}/identificationField", payment_path(payment_id)?);
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn get_payment_status(
        &self,
        payment_id: &str,
    ) -> Result<PaymentStatusResponse, Error> {
        let path = format!("{}/status", payment_path(payment_id)?);
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn get_payment_billing_info(
        &self,
        payment_id: &str,
    ) -> Result<BillingInfoResponse, Error> {
        let path = format!("{}/billingInfo", payment_path(payment_id)?);
        self.send_typed::<(), _>(Method::GET, &path, None).await
    }

    pub async fn list_lifecycle_payments(
        &self,
        request: &LifecyclePaymentListRequest,
    ) -> Result<LifecyclePaymentListResponse, LifecycleError> {
        let mut path = "/v3/payments".to_string();
        let mut has_query = false;
        if let Some(value) = request.offset {
            push_query(&mut path, &mut has_query, "offset", &value.to_string());
        }
        if let Some(value) = request.limit {
            push_query(&mut path, &mut has_query, "limit", &value.to_string());
        }
        if let Some(value) = request.customer.as_deref() {
            push_query(&mut path, &mut has_query, "customer", value);
        }
        if let Some(value) = request.external_reference.as_deref() {
            push_query(&mut path, &mut has_query, "externalReference", value);
        }
        self.send_lifecycle_typed(Method::GET, &path).await
    }

    pub async fn get_lifecycle_payment(
        &self,
        payment_id: &str,
    ) -> Result<LifecyclePaymentResponse, LifecycleError> {
        let payment_id = encode_identifier(payment_id).ok_or(LifecycleError::InvalidIdentifier)?;
        self.send_lifecycle_typed(Method::GET, &format!("/v3/payments/{payment_id}"))
            .await
    }

    pub async fn cancel_lifecycle_payment(
        &self,
        payment_id: &str,
    ) -> Result<LifecyclePaymentCancellationResponse, LifecycleError> {
        let payment_id = encode_identifier(payment_id).ok_or(LifecycleError::InvalidIdentifier)?;
        self.send_lifecycle_typed(Method::DELETE, &format!("/v3/payments/{payment_id}"))
            .await
    }
}

impl Client {
    /// Read the legacy payment summary with exact decimal tokens.
    /// Use `get_payment_details` for reconciliation that must account for refunds.
    pub async fn get_payment(
        &self,
        payment_id: &str,
    ) -> Result<crate::types::PaymentResponse, Error> {
        self.send_typed::<(), _>(Method::GET, &payment_path(payment_id)?, None)
            .await
    }

    /// Read the current payment and every refund record in one authenticated request.
    pub async fn get_payment_details(
        &self,
        payment_id: &str,
    ) -> Result<crate::types::PaymentDetails, Error> {
        self.send_typed::<(), _>(Method::GET, &payment_path(payment_id)?, None)
            .await
    }

    /// Identify the authenticated receiving account without private financial APIs.
    pub async fn get_wallets(&self) -> Result<crate::types::WalletsResponse, Error> {
        self.send_typed::<(), _>(Method::GET, "/v3/wallets/", None)
            .await
    }
}

fn payment_path(payment_id: &str) -> Result<String, Error> {
    let encoded = encode_identifier(payment_id).ok_or(Error::InvalidIdentifier)?;
    Ok(format!("/v3/payments/{encoded}"))
}

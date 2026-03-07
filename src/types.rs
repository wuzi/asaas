use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BillingType {
    Undefined,
    Boleto,
    CreditCard,
    Pix,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeanPaymentCreateRequest {
    pub customer: String,
    pub billing_type: BillingType,
    pub value: f64,
    pub due_date: String,
    pub description: Option<String>,
    pub days_after_due_date_to_registration_cancellation: Option<i32>,
    pub external_reference: Option<String>,
    pub installment_count: Option<i32>,
    pub total_value: Option<f64>,
    pub installment_value: Option<f64>,
    pub discount: Option<PaymentDiscount>,
    pub interest: Option<PaymentInterest>,
    pub fine: Option<PaymentFine>,
    pub postal_service: Option<bool>,
    pub split: Option<Vec<PaymentSplitItem>>,
    pub callback: Option<PaymentCallback>,
    pub pix_automatic_authorization_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDiscount {
    pub value: f64,
    pub due_date_limit_days: Option<i32>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInterest {
    pub value: Option<f64>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentFine {
    pub value: Option<f64>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentSplitItem {
    pub wallet_id: String,
    pub fixed_value: Option<f64>,
    pub percentual_value: Option<f64>,
    pub total_fixed_value: Option<f64>,
    pub external_reference: Option<String>,
    pub description: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentCallback {
    pub success_url: String,
    pub auto_redirect: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeanPaymentResponse {
    pub object: String,
    pub id: String,
    pub date_created: Option<String>,
    pub customer_id: Option<String>,
    pub subscription_id: Option<String>,
    pub installment_id: Option<String>,
    pub payment_link_id: Option<String>,
    pub value: Option<f64>,
    pub net_value: Option<f64>,
    pub original_value: Option<f64>,
    pub interest_value: Option<f64>,
    pub description: Option<String>,
    pub billing_type: Option<BillingType>,
    pub can_be_paid_after_due_date: Option<bool>,
    pub confirmed_date: Option<String>,
    pub pix_transaction_id: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<String>,
    pub original_due_date: Option<String>,
    pub payment_date: Option<String>,
    pub customer_payment_date: Option<String>,
    pub installment_number: Option<i32>,
    pub external_reference: Option<String>,
    pub deleted: Option<bool>,
    pub anticipated: Option<bool>,
    pub anticipable: Option<bool>,
    pub credit_date: Option<String>,
    pub transaction_receipt_url: Option<String>,
    pub duplicated_payment_id: Option<String>,
    pub discount: Option<PaymentDiscount>,
    pub fine: Option<PaymentValueField>,
    pub interest: Option<PaymentValueField>,
    pub postal_service: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentValueField {
    pub value: f64,
}

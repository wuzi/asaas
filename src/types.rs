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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Pending,
    Received,
    Confirmed,
    Overdue,
    Refunded,
    ReceivedInCash,
    RefundRequested,
    RefundInProgress,
    ChargebackRequested,
    ChargebackDispute,
    AwaitingChargebackReversal,
    DunningRequested,
    DunningReceived,
    AwaitingRiskAnalysis,
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
    pub installment_count: Option<usize>,
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
pub struct CreateCustomerRequest {
    pub name: String,
    pub cpf_cnpj: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile_phone: Option<String>,
    pub address: Option<String>,
    pub address_number: Option<String>,
    pub complement: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub external_reference: Option<String>,
    pub notification_disabled: Option<bool>,
    pub additional_emails: Option<String>,
    pub municipal_inscription: Option<String>,
    pub state_inscription: Option<String>,
    pub observations: Option<String>,
    pub group_name: Option<String>,
    pub company: Option<String>,
    pub foreign_customer: Option<bool>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerResponse {
    pub object: String,
    pub id: String,
    pub date_created: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub company: Option<String>,
    pub phone: Option<String>,
    pub mobile_phone: Option<String>,
    pub address: Option<String>,
    pub address_number: Option<String>,
    pub complement: Option<String>,
    pub province: Option<String>,
    pub postal_code: Option<String>,
    pub cpf_cnpj: Option<String>,
    pub person_type: Option<String>,
    pub deleted: Option<bool>,
    pub additional_emails: Option<String>,
    pub external_reference: Option<String>,
    pub notification_disabled: Option<bool>,
    pub observations: Option<String>,
    pub municipal_inscription: Option<String>,
    pub state_inscription: Option<String>,
    pub can_delete: Option<bool>,
    pub cannot_be_deleted_reason: Option<String>,
    pub can_edit: Option<bool>,
    pub cannot_edit_reason: Option<String>,
    pub city: Option<i64>,
    pub city_name: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDiscount {
    pub value: f64,
    pub limit_date: Option<String>,
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
    #[serde(alias = "customer")]
    pub customer_id: Option<String>,
    #[serde(alias = "subscription")]
    pub subscription_id: Option<String>,
    #[serde(alias = "installment")]
    pub installment_id: Option<String>,
    #[serde(alias = "checkoutSession")]
    pub checkout_session_id: Option<String>,
    #[serde(alias = "paymentLink")]
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
    pub status: Option<PaymentStatus>,
    pub due_date: Option<String>,
    pub original_due_date: Option<String>,
    pub payment_date: Option<String>,
    #[serde(alias = "clientPaymentDate")]
    pub customer_payment_date: Option<String>,
    pub installment_number: Option<i32>,
    pub invoice_url: Option<String>,
    pub invoice_number: Option<String>,
    pub external_reference: Option<String>,
    pub deleted: Option<bool>,
    pub anticipated: Option<bool>,
    pub anticipable: Option<bool>,
    pub credit_date: Option<String>,
    pub estimated_credit_date: Option<String>,
    pub transaction_receipt_url: Option<String>,
    pub nosso_numero: Option<String>,
    pub bank_slip_url: Option<String>,
    pub last_invoice_viewed_date: Option<String>,
    pub last_bank_slip_viewed_date: Option<String>,
    pub duplicated_payment_id: Option<String>,
    pub days_after_due_date_to_registration_cancellation: Option<i32>,
    pub discount: Option<PaymentDiscount>,
    pub fine: Option<PaymentValueField>,
    pub interest: Option<PaymentValueField>,
    pub postal_service: Option<bool>,
    pub escrow: Option<serde_json::Value>,
    pub refunds: Option<serde_json::Value>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentValueField {
    pub value: f64,
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixQrCodeResponse {
    pub success: bool,
    pub encoded_image: String,
    pub payload: String,
    pub expiration_date: Option<String>,
    pub description: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentificationFieldResponse {
    pub identification_field: String,
    pub nosso_numero: Option<String>,
    pub bar_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentStatusResponse {
    pub status: PaymentStatus,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallmentPaymentsListResponse {
    pub object: String,
    pub has_more: bool,
    pub total_count: u64,
    pub limit: u64,
    pub offset: u64,
    pub data: Vec<LeanPaymentResponse>,
}

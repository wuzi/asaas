use reqwest::Method;

use crate::client::{Client, encode_url_component, push_query};
use crate::error::{Error, LifecycleError};
use crate::types::{
    CreateCustomerRequest, CustomerListRequest, CustomerListResponse, CustomerResponse,
};

impl Client {
    pub async fn create_customer(
        &self,
        payload: &CreateCustomerRequest,
    ) -> Result<CustomerResponse, Error> {
        self.send_typed(Method::POST, "/v3/customers", Some(payload))
            .await
    }

    pub async fn list_customers(
        &self,
        request: &CustomerListRequest,
    ) -> Result<CustomerListResponse, LifecycleError> {
        let mut path = "/v3/customers".to_string();
        let mut has_query = false;
        if let Some(value) = request.offset {
            push_query(&mut path, &mut has_query, "offset", &value.to_string());
        }
        if let Some(value) = request.limit {
            push_query(&mut path, &mut has_query, "limit", &value.to_string());
        }
        if let Some(value) = request.cpf_cnpj.as_deref() {
            push_query(&mut path, &mut has_query, "cpfCnpj", value);
        }
        if let Some(value) = request.external_reference.as_deref() {
            push_query(&mut path, &mut has_query, "externalReference", value);
        }
        self.send_lifecycle_typed(Method::GET, &path).await
    }

    pub async fn get_customer(
        &self,
        customer_id: &str,
    ) -> Result<CustomerResponse, LifecycleError> {
        let customer_id = encode_url_component(customer_id);
        self.send_lifecycle_typed(Method::GET, &format!("/v3/customers/{customer_id}"))
            .await
    }
}

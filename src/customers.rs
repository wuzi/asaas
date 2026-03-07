use reqwest::Method;

use crate::client::Client;
use crate::error::Error;
use crate::types::{CreateCustomerRequest, CustomerResponse};

impl Client {
    pub async fn create_customer(
        &self,
        payload: &CreateCustomerRequest,
    ) -> Result<CustomerResponse, Error> {
        self.send_typed(Method::POST, "/v3/customers", Some(payload))
            .await
    }
}

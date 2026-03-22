use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BASE_URL: &str = "https://api.cardknox.com/v2";
const API_VERSION: &str = "2.1";

/// A customer record as returned by the Sola Recurring API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GatewayCustomer {
    #[serde(default)]
    pub customer_id: String,
    #[serde(default)]
    pub customer_number: String,
    #[serde(default)]
    pub bill_first_name: String,
    #[serde(default)]
    pub bill_last_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub bill_phone: String,
    #[serde(default)]
    pub bill_street: String,
    #[serde(default)]
    pub bill_city: String,
    #[serde(default)]
    pub bill_state: String,
    #[serde(default)]
    pub bill_zip: String,
    #[serde(default)]
    pub bill_country: String,
    #[serde(default)]
    pub revision: i64,
}

/// Response from ListCustomers endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListCustomersResponse {
    #[serde(default)]
    pub customers: Vec<GatewayCustomer>,
    #[serde(default)]
    pub next_token: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub total_count: Option<i64>,
}

/// Response from CreateCustomer endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateCustomerResponse {
    #[serde(default)]
    pub customer_id: String,
    #[serde(default)]
    pub revision: i64,
}

/// Response from UpdateCustomer endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateCustomerResponse {
    #[serde(default)]
    pub revision: i64,
}

/// Response from DeleteCustomer endpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct DeleteCustomerResponse {
    #[serde(default)]
    pub status: String,
}

/// Typed errors from API interactions
#[derive(Debug)]
pub enum RecurringApiError {
    /// HTTP 401/403 - invalid or expired API key
    Unauthorized(String),
    /// Network error or timeout
    NetworkError(String),
    /// API returned an error response
    ApiError { status: u16, message: String },
    /// Failed to parse response
    ParseError(String),
}

impl std::fmt::Display for RecurringApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecurringApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            RecurringApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            RecurringApiError::ApiError { status, message } => {
                write!(f, "API error ({}): {}", status, message)
            }
            RecurringApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

/// Client for the Sola Recurring API v2
pub struct RecurringApiClient {
    api_key: String,
    http_client: reqwest::Client,
}

impl RecurringApiClient {
    pub fn new(api_key: String) -> Result<Self, RecurringApiError> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| RecurringApiError::NetworkError(e.to_string()))?;

        Ok(Self {
            api_key,
            http_client,
        })
    }

    /// Make a POST request to the Sola Recurring API
    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &HashMap<String, serde_json::Value>,
    ) -> Result<T, RecurringApiError> {
        let url = format!("{}{}", BASE_URL, endpoint);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", &self.api_key)
            .header("X-Recurring-Api-Version", API_VERSION)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    RecurringApiError::NetworkError("Request timed out".to_string())
                } else if e.is_connect() {
                    RecurringApiError::NetworkError(
                        "Could not connect to Sola API".to_string(),
                    )
                } else {
                    RecurringApiError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status().as_u16();

        if status == 401 || status == 403 {
            let body_text = response.text().await.unwrap_or_default();
            return Err(RecurringApiError::Unauthorized(format!(
                "Invalid or expired API key (HTTP {}): {}",
                status, body_text
            )));
        }

        let body_text = response.text().await.map_err(|e| {
            RecurringApiError::ParseError(format!("Failed to read response body: {}", e))
        })?;

        if status >= 400 {
            return Err(RecurringApiError::ApiError {
                status,
                message: body_text,
            });
        }

        serde_json::from_str::<T>(&body_text).map_err(|e| {
            RecurringApiError::ParseError(format!(
                "Failed to parse response: {}. Body: {}",
                e, body_text
            ))
        })
    }

    /// List all customers from the gateway, handling pagination
    pub async fn list_all_customers(&self) -> Result<Vec<GatewayCustomer>, RecurringApiError> {
        let mut all_customers = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut body = HashMap::new();
            body.insert(
                "SoftwareName".to_string(),
                serde_json::Value::String("Vira".to_string()),
            );
            body.insert(
                "SoftwareVersion".to_string(),
                serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
            );
            body.insert(
                "PageSize".to_string(),
                serde_json::Value::Number(serde_json::Number::from(100)),
            );

            if let Some(ref token) = next_token {
                body.insert(
                    "NextToken".to_string(),
                    serde_json::Value::String(token.clone()),
                );
            }

            let response: ListCustomersResponse = self.post("/ListCustomers", &body).await?;

            all_customers.extend(response.customers);

            match response.next_token {
                Some(ref token) if !token.is_empty() => {
                    next_token = Some(token.clone());
                }
                _ => break,
            }
        }

        Ok(all_customers)
    }

    /// Create a new customer in the gateway
    pub async fn create_customer(
        &self,
        customer_number: &str,
        first_name: &str,
        last_name: &str,
        email: &str,
        phone: &str,
        street: &str,
        city: &str,
        state: &str,
        zip: &str,
        country: &str,
    ) -> Result<CreateCustomerResponse, RecurringApiError> {
        let mut body = HashMap::new();
        body.insert(
            "SoftwareName".to_string(),
            serde_json::Value::String("Vira".to_string()),
        );
        body.insert(
            "SoftwareVersion".to_string(),
            serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
        );
        body.insert(
            "CustomerNumber".to_string(),
            serde_json::Value::String(customer_number.to_string()),
        );
        body.insert(
            "BillFirstName".to_string(),
            serde_json::Value::String(first_name.to_string()),
        );
        body.insert(
            "BillLastName".to_string(),
            serde_json::Value::String(last_name.to_string()),
        );
        body.insert(
            "Email".to_string(),
            serde_json::Value::String(email.to_string()),
        );
        body.insert(
            "BillPhone".to_string(),
            serde_json::Value::String(phone.to_string()),
        );
        body.insert(
            "BillStreet".to_string(),
            serde_json::Value::String(street.to_string()),
        );
        body.insert(
            "BillCity".to_string(),
            serde_json::Value::String(city.to_string()),
        );
        body.insert(
            "BillState".to_string(),
            serde_json::Value::String(state.to_string()),
        );
        body.insert(
            "BillZip".to_string(),
            serde_json::Value::String(zip.to_string()),
        );
        body.insert(
            "BillCountry".to_string(),
            serde_json::Value::String(country.to_string()),
        );

        self.post("/CreateCustomer", &body).await
    }

    /// Update an existing customer in the gateway (ALL fields required per API spec)
    pub async fn update_customer(
        &self,
        customer_id: &str,
        customer_number: &str,
        first_name: &str,
        last_name: &str,
        email: &str,
        phone: &str,
        street: &str,
        city: &str,
        state: &str,
        zip: &str,
        country: &str,
        revision: i64,
    ) -> Result<UpdateCustomerResponse, RecurringApiError> {
        let mut body = HashMap::new();
        body.insert(
            "SoftwareName".to_string(),
            serde_json::Value::String("Vira".to_string()),
        );
        body.insert(
            "SoftwareVersion".to_string(),
            serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
        );
        body.insert(
            "CustomerId".to_string(),
            serde_json::Value::String(customer_id.to_string()),
        );
        body.insert(
            "CustomerNumber".to_string(),
            serde_json::Value::String(customer_number.to_string()),
        );
        body.insert(
            "BillFirstName".to_string(),
            serde_json::Value::String(first_name.to_string()),
        );
        body.insert(
            "BillLastName".to_string(),
            serde_json::Value::String(last_name.to_string()),
        );
        body.insert(
            "Email".to_string(),
            serde_json::Value::String(email.to_string()),
        );
        body.insert(
            "BillPhone".to_string(),
            serde_json::Value::String(phone.to_string()),
        );
        body.insert(
            "BillStreet".to_string(),
            serde_json::Value::String(street.to_string()),
        );
        body.insert(
            "BillCity".to_string(),
            serde_json::Value::String(city.to_string()),
        );
        body.insert(
            "BillState".to_string(),
            serde_json::Value::String(state.to_string()),
        );
        body.insert(
            "BillZip".to_string(),
            serde_json::Value::String(zip.to_string()),
        );
        body.insert(
            "BillCountry".to_string(),
            serde_json::Value::String(country.to_string()),
        );
        body.insert(
            "Revision".to_string(),
            serde_json::Value::Number(serde_json::Number::from(revision)),
        );

        self.post("/UpdateCustomer", &body).await
    }

    /// Delete a customer from the gateway
    pub async fn delete_customer(
        &self,
        customer_id: &str,
    ) -> Result<DeleteCustomerResponse, RecurringApiError> {
        let mut body = HashMap::new();
        body.insert(
            "SoftwareName".to_string(),
            serde_json::Value::String("Vira".to_string()),
        );
        body.insert(
            "SoftwareVersion".to_string(),
            serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
        );
        body.insert(
            "CustomerId".to_string(),
            serde_json::Value::String(customer_id.to_string()),
        );

        self.post("/DeleteCustomer", &body).await
    }
}

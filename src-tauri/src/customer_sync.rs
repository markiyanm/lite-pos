use crate::recurring_api::{GatewayCustomer, RecurringApiClient, RecurringApiError};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global lock to prevent concurrent sync operations
static SYNC_RUNNING: AtomicBool = AtomicBool::new(false);

/// Request payload for creating a customer on the gateway
#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub customer_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

/// Request payload for updating a customer on the gateway
#[derive(Debug, Deserialize)]
pub struct UpdateCustomerRequest {
    pub customer_id: String,
    pub customer_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub revision: i64,
}

/// Result returned for each gateway operation
#[derive(Debug, Serialize, Clone)]
pub struct SyncOperationResult {
    pub success: bool,
    pub customer_id: Option<String>,
    pub revision: Option<i64>,
    pub error: Option<String>,
    pub error_type: Option<String>,
}

/// Result of listing all gateway customers
#[derive(Debug, Serialize)]
pub struct ListGatewayCustomersResult {
    pub success: bool,
    pub customers: Vec<GatewayCustomerData>,
    pub error: Option<String>,
    pub error_type: Option<String>,
}

/// Serializable gateway customer data for frontend
#[derive(Debug, Serialize, Clone)]
pub struct GatewayCustomerData {
    pub customer_id: String,
    pub customer_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub revision: i64,
}

impl From<GatewayCustomer> for GatewayCustomerData {
    fn from(c: GatewayCustomer) -> Self {
        Self {
            customer_id: c.customer_id,
            customer_number: c.customer_number,
            first_name: c.bill_first_name,
            last_name: c.bill_last_name,
            email: c.email,
            phone: c.bill_phone,
            street: c.bill_street,
            city: c.bill_city,
            state: c.bill_state,
            zip: c.bill_zip,
            country: c.bill_country,
            revision: c.revision,
        }
    }
}

fn error_type_string(err: &RecurringApiError) -> String {
    match err {
        RecurringApiError::Unauthorized(_) => "unauthorized".to_string(),
        RecurringApiError::NetworkError(_) => "network".to_string(),
        RecurringApiError::ApiError { .. } => "api".to_string(),
        RecurringApiError::ParseError(_) => "parse".to_string(),
    }
}

/// Acquire the sync lock. Returns true if acquired, false if already running.
#[tauri::command]
pub fn gateway_sync_acquire_lock() -> bool {
    SYNC_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

/// Release the sync lock.
#[tauri::command]
pub fn gateway_sync_release_lock() {
    SYNC_RUNNING.store(false, Ordering::SeqCst);
}

/// Check if sync is currently running
#[tauri::command]
pub fn gateway_sync_is_running() -> bool {
    SYNC_RUNNING.load(Ordering::SeqCst)
}

/// List all customers from the gateway
#[tauri::command]
pub async fn gateway_list_customers(
    api_key: String,
) -> Result<ListGatewayCustomersResult, String> {
    // Never log the API key
    log::info!("[CustomerSync] Listing gateway customers");

    let client = RecurringApiClient::new(api_key).map_err(|e| e.to_string())?;

    match client.list_all_customers().await {
        Ok(customers) => {
            log::info!(
                "[CustomerSync] Listed {} gateway customers",
                customers.len()
            );
            Ok(ListGatewayCustomersResult {
                success: true,
                customers: customers.into_iter().map(|c| c.into()).collect(),
                error: None,
                error_type: None,
            })
        }
        Err(e) => {
            log::error!("[CustomerSync] Failed to list gateway customers: {}", e);
            Ok(ListGatewayCustomersResult {
                success: false,
                customers: Vec::new(),
                error: Some(e.to_string()),
                error_type: Some(error_type_string(&e)),
            })
        }
    }
}

/// Create a customer on the gateway
#[tauri::command]
pub async fn gateway_create_customer(
    api_key: String,
    customer: CreateCustomerRequest,
) -> Result<SyncOperationResult, String> {
    log::info!(
        "[CustomerSync] Creating gateway customer for CustomerNumber={}",
        customer.customer_number
    );

    let client = RecurringApiClient::new(api_key).map_err(|e| e.to_string())?;

    match client
        .create_customer(
            &customer.customer_number,
            &customer.first_name,
            &customer.last_name,
            &customer.email,
            &customer.phone,
            &customer.street,
            &customer.city,
            &customer.state,
            &customer.zip,
            &customer.country,
        )
        .await
    {
        Ok(resp) => {
            log::info!(
                "[CustomerSync] Created gateway customer: CustomerId={}",
                resp.customer_id
            );
            Ok(SyncOperationResult {
                success: true,
                customer_id: Some(resp.customer_id),
                revision: Some(resp.revision),
                error: None,
                error_type: None,
            })
        }
        Err(e) => {
            log::error!("[CustomerSync] Failed to create gateway customer: {}", e);
            Ok(SyncOperationResult {
                success: false,
                customer_id: None,
                revision: None,
                error: Some(e.to_string()),
                error_type: Some(error_type_string(&e)),
            })
        }
    }
}

/// Update a customer on the gateway
#[tauri::command]
pub async fn gateway_update_customer(
    api_key: String,
    customer: UpdateCustomerRequest,
) -> Result<SyncOperationResult, String> {
    log::info!(
        "[CustomerSync] Updating gateway customer: CustomerId={}",
        customer.customer_id
    );

    let client = RecurringApiClient::new(api_key).map_err(|e| e.to_string())?;

    match client
        .update_customer(
            &customer.customer_id,
            &customer.customer_number,
            &customer.first_name,
            &customer.last_name,
            &customer.email,
            &customer.phone,
            &customer.street,
            &customer.city,
            &customer.state,
            &customer.zip,
            &customer.country,
            customer.revision,
        )
        .await
    {
        Ok(resp) => {
            log::info!(
                "[CustomerSync] Updated gateway customer, new revision={}",
                resp.revision
            );
            Ok(SyncOperationResult {
                success: true,
                customer_id: Some(customer.customer_id),
                revision: Some(resp.revision),
                error: None,
                error_type: None,
            })
        }
        Err(e) => {
            log::error!("[CustomerSync] Failed to update gateway customer: {}", e);
            Ok(SyncOperationResult {
                success: false,
                customer_id: None,
                revision: None,
                error: Some(e.to_string()),
                error_type: Some(error_type_string(&e)),
            })
        }
    }
}

/// Delete a customer from the gateway
#[tauri::command]
pub async fn gateway_delete_customer(
    api_key: String,
    customer_id: String,
) -> Result<SyncOperationResult, String> {
    log::info!(
        "[CustomerSync] Deleting gateway customer: CustomerId={}",
        customer_id
    );

    let client = RecurringApiClient::new(api_key).map_err(|e| e.to_string())?;

    match client.delete_customer(&customer_id).await {
        Ok(_resp) => {
            log::info!("[CustomerSync] Deleted gateway customer");
            Ok(SyncOperationResult {
                success: true,
                customer_id: Some(customer_id),
                revision: None,
                error: None,
                error_type: None,
            })
        }
        Err(e) => {
            log::error!("[CustomerSync] Failed to delete gateway customer: {}", e);
            Ok(SyncOperationResult {
                success: false,
                customer_id: None,
                revision: None,
                error: Some(e.to_string()),
                error_type: Some(error_type_string(&e)),
            })
        }
    }
}

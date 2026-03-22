use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolaTransactionRequest {
    #[serde(rename = "xKey")]
    pub x_key: String,
    #[serde(rename = "xSoftwareName")]
    pub x_software_name: String,
    #[serde(rename = "xSoftwareVersion")]
    pub x_software_version: String,
    #[serde(rename = "xExternalRequestId")]
    pub x_external_request_id: String,
    #[serde(rename = "xCommand")]
    pub x_command: String,
    #[serde(rename = "xAmount")]
    pub x_amount: String,
    #[serde(rename = "xDeviceId")]
    pub x_device_id: String,
    #[serde(rename = "xInvoice", skip_serializing_if = "Option::is_none")]
    pub x_invoice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolaTransactionResponse {
    #[serde(rename = "xRefnum")]
    pub x_ref_num: String,
    #[serde(rename = "xResult")]
    pub x_result: String,
    #[serde(rename = "xStatus")]
    pub x_status: String,
    #[serde(rename = "xAuthCode")]
    pub x_auth_code: Option<String>,
    #[serde(rename = "xMaskedCardNumber")]
    pub x_masked_card_number: Option<String>,
    #[serde(rename = "xCardType")]
    pub x_card_type: Option<String>,
    #[serde(rename = "xName")]
    pub x_name: Option<String>,
    #[serde(rename = "xExp")]
    pub x_exp: Option<String>,
    #[serde(rename = "xAvsResult")]
    pub x_avs_result: Option<String>,
    #[serde(rename = "xCvvResult")]
    pub x_cvv_result: Option<String>,
    #[serde(rename = "xError")]
    pub x_error: Option<String>,
    #[serde(rename = "xAmt")]
    pub x_amt: Option<String>,
    #[serde(rename = "xEntryMethod")]
    pub x_entry_method: Option<String>,
}

/// Serializable request info for frontend debugging display (API key masked)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolaRequestInfo {
    pub url: String,
    pub command: String,
    pub amount: String,
    pub device_id: String,
    pub invoice: Option<String>,
    pub request_id: String,
    pub masked_key: String,
    pub software_name: String,
    pub software_version: String,
    pub timestamp: String,
}

/// Combined result returned from transaction commands
#[derive(Debug, Serialize, Deserialize)]
pub struct SolaTransactionResult {
    pub request_info: SolaRequestInfo,
    pub response: SolaTransactionResponse,
    pub raw_response: String,
    pub http_status: u16,
    pub duration_ms: u64,
}

fn build_request_info(
    api_key: &str,
    command: &str,
    amount: &str,
    device_id: &str,
    invoice: &Option<String>,
    request_id: &str,
) -> SolaRequestInfo {
    let masked_key = if api_key.len() > 10 {
        format!("{}...{}", &api_key[..5], &api_key[api_key.len() - 5..])
    } else {
        "***".to_string()
    };

    SolaRequestInfo {
        url: "https://device.cardknox.com/v2/session".to_string(),
        command: command.to_string(),
        amount: amount.to_string(),
        device_id: device_id.to_string(),
        invoice: invoice.clone(),
        request_id: request_id.to_string(),
        masked_key,
        software_name: "MM Lite POS".to_string(),
        software_version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: {
            let duration = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = duration.as_secs();
            let millis = duration.subsec_millis();
            format!("{}.{:03}", secs, millis)
        },
    }
}

/// Build the request info and return it for the frontend to display immediately.
/// This is called BEFORE the actual HTTP request.
#[tauri::command]
pub fn build_sola_request_info(
    api_key: String,
    device_id: String,
    amount_cents: i64,
    invoice_number: Option<String>,
    command: Option<String>,
) -> Result<SolaRequestInfo, String> {
    let amount_dollars = format!("{:.2}", amount_cents as f64 / 100.0);
    let request_id = uuid::Uuid::new_v4().to_string();
    let cmd = command.unwrap_or_else(|| "cc:sale".to_string());

    Ok(build_request_info(
        &api_key,
        &cmd,
        &amount_dollars,
        &device_id,
        &invoice_number,
        &request_id,
    ))
}

#[tauri::command]
pub async fn process_sola_transaction(
    api_key: String,
    device_id: String,
    amount_cents: i64,
    invoice_number: Option<String>,
) -> Result<SolaTransactionResult, String> {
    execute_sola_command(&api_key, &device_id, amount_cents, &invoice_number, "cc:sale").await
}

#[tauri::command]
pub async fn cancel_sola_transaction(
    api_key: String,
    device_id: String,
) -> Result<SolaTransactionResult, String> {
    execute_sola_command(&api_key, &device_id, 0, &None, "cc:cancel").await
}

async fn execute_sola_command(
    api_key: &str,
    device_id: &str,
    amount_cents: i64,
    invoice_number: &Option<String>,
    command: &str,
) -> Result<SolaTransactionResult, String> {
    let amount_dollars = format!("{:.2}", amount_cents as f64 / 100.0);
    let request_id = uuid::Uuid::new_v4().to_string();

    let request = SolaTransactionRequest {
        x_key: api_key.to_string(),
        x_software_name: "MM Lite POS".to_string(),
        x_software_version: env!("CARGO_PKG_VERSION").to_string(),
        x_external_request_id: request_id.clone(),
        x_command: command.to_string(),
        x_amount: amount_dollars.clone(),
        x_device_id: device_id.to_string(),
        x_invoice: invoice_number.clone(),
    };

    let request_info = build_request_info(
        api_key,
        command,
        &amount_dollars,
        device_id,
        invoice_number,
        &request_id,
    );

    // Log request details
    println!("[Sola] Transaction Request:");
    println!("[Sola]   URL: {}", request_info.url);
    println!("[Sola]   xKey: {}", request_info.masked_key);
    println!("[Sola]   xCommand: {}", command);
    println!("[Sola]   xAmount: {}", amount_dollars);
    println!("[Sola]   xDeviceId: {}", device_id);
    println!("[Sola]   xExternalRequestId: {}", request_id);
    println!("[Sola]   xInvoice: {:?}", invoice_number);

    let start = std::time::Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post("https://device.cardknox.com/v2/session")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            println!("[Sola] Request Error: {:?}", e);
            if e.is_timeout() {
                "Transaction timed out. Please check the device and try again.".to_string()
            } else if e.is_connect() {
                "Could not connect to Sola Gateway. Check your internet connection.".to_string()
            } else {
                format!("Network error: {}", e)
            }
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let http_status = response.status().as_u16();
    println!("[Sola] Response Status: {} ({}ms)", http_status, duration_ms);

    let response_text = response.text().await.map_err(|e| {
        println!("[Sola] Failed to read response body: {}", e);
        format!("Failed to read response: {}", e)
    })?;

    println!("[Sola] Response Body: {}", response_text);

    if http_status >= 400 {
        return Err(format!(
            "HTTP error {}: {}",
            http_status, response_text
        ));
    }

    let sola_response: SolaTransactionResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("[Sola] Failed to parse response JSON: {}", e);
            format!("Failed to parse response: {}. Body: {}", e, response_text)
        })?;

    println!("[Sola] Parsed Response: {:?}", sola_response);

    Ok(SolaTransactionResult {
        request_info,
        response: sola_response,
        raw_response: response_text,
        http_status,
        duration_ms,
    })
}

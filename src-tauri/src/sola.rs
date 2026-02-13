use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[tauri::command]
pub async fn process_sola_transaction(
    api_key: String,
    device_id: String,
    amount_cents: i64,
    invoice_number: Option<String>,
) -> Result<SolaTransactionResponse, String> {
    // Convert cents to dollars with 2 decimal places
    let amount_dollars = format!("{:.2}", amount_cents as f64 / 100.0);

    // Generate unique request ID
    let request_id = uuid::Uuid::new_v4().to_string();

    let request = SolaTransactionRequest {
        x_key: api_key.clone(),
        x_software_name: "MM Lite POS".to_string(),
        x_software_version: env!("CARGO_PKG_VERSION").to_string(),
        x_external_request_id: request_id.clone(),
        x_command: "cc:sale".to_string(),
        x_amount: amount_dollars.clone(),
        x_device_id: device_id.clone(),
        x_invoice: invoice_number.clone(),
    };

    // Log request details (mask API key)
    let masked_key = if api_key.len() > 10 {
        format!("{}...{}", &api_key[..5], &api_key[api_key.len()-5..])
    } else {
        "***".to_string()
    };
    println!("[Sola] Transaction Request:");
    println!("[Sola]   URL: https://device.cardknox.com/v2/session");
    println!("[Sola]   xKey: {}", masked_key);
    println!("[Sola]   xSoftwareName: MM Lite POS");
    println!("[Sola]   xSoftwareVersion: {}", env!("CARGO_PKG_VERSION"));
    println!("[Sola]   xExternalRequestId: {}", request_id);
    println!("[Sola]   xCommand: cc:sale");
    println!("[Sola]   xAmount: {}", amount_dollars);
    println!("[Sola]   xDeviceId: {}", device_id);
    println!("[Sola]   xInvoice: {:?}", invoice_number);

    // Use reqwest for async HTTP
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120)) // 2 minute timeout for card interaction
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

    let status = response.status();
    println!("[Sola] Response Status: {}", status);

    // Get response body as text first for logging
    let response_text = response.text().await.map_err(|e| {
        println!("[Sola] Failed to read response body: {}", e);
        format!("Failed to read response: {}", e)
    })?;

    println!("[Sola] Response Body: {}", response_text);

    if !status.is_success() {
        return Err(format!(
            "HTTP error {}: {}. Response: {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown"),
            response_text
        ));
    }

    let sola_response: SolaTransactionResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("[Sola] Failed to parse response JSON: {}", e);
            format!("Failed to parse response: {}. Body: {}", e, response_text)
        })?;

    println!("[Sola] Parsed Response: {:?}", sola_response);
    Ok(sola_response)
}

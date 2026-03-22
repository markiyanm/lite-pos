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
    #[serde(rename = "xToken")]
    pub x_token: Option<String>,
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

/// CNP request info for frontend debugging (no card data exposed)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolaCnpRequestInfo {
    pub url: String,
    pub command: String,
    pub amount: String,
    pub invoice: Option<String>,
    pub request_id: String,
    pub masked_key: String,
    pub software_name: String,
    pub software_version: String,
    pub timestamp: String,
}

/// CNP transaction result
#[derive(Debug, Serialize, Deserialize)]
pub struct SolaCnpTransactionResult {
    pub request_info: SolaCnpRequestInfo,
    pub response: SolaTransactionResponse,
    pub raw_response: String,
    pub http_status: u16,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn process_sola_cnp_transaction(
    api_key: String,
    card_token: String,
    cvv_token: String,
    exp: String,
    amount_cents: i64,
    invoice_number: Option<String>,
    name: Option<String>,
    zip: Option<String>,
) -> Result<SolaCnpTransactionResult, String> {
    let amount_dollars = format!("{:.2}", amount_cents as f64 / 100.0);
    let request_id = uuid::Uuid::new_v4().to_string();
    let version = env!("CARGO_PKG_VERSION");

    let masked_key = if api_key.len() > 10 {
        format!("{}...{}", &api_key[..5], &api_key[api_key.len() - 5..])
    } else {
        "***".to_string()
    };

    let request_info = SolaCnpRequestInfo {
        url: "https://x1.cardknox.com/gateway".to_string(),
        command: "cc:sale".to_string(),
        amount: amount_dollars.clone(),
        invoice: invoice_number.clone(),
        request_id: request_id.clone(),
        masked_key,
        software_name: "Vira".to_string(),
        software_version: version.to_string(),
        timestamp: {
            let duration = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
        },
    };

    // Log request (NO card data)
    println!("[Sola CNP] Transaction Request:");
    println!("[Sola CNP]   URL: {}", request_info.url);
    println!("[Sola CNP]   xKey: {}", request_info.masked_key);
    println!("[Sola CNP]   xCommand: cc:sale");
    println!("[Sola CNP]   xAmount: {}", amount_dollars);
    println!("[Sola CNP]   xInvoice: {:?}", invoice_number);
    // NOTE: card_token and cvv_token are NOT logged for PCI compliance

    let start = std::time::Instant::now();

    // Build form-encoded body
    let mut params: Vec<(&str, String)> = vec![
        ("xKey", api_key),
        ("xVersion", "5.0.0".to_string()),
        ("xSoftwareName", "Vira".to_string()),
        ("xSoftwareVersion", version.to_string()),
        ("xCommand", "cc:sale".to_string()),
        ("xAmount", amount_dollars),
        ("xCardNum", card_token),
        ("xCVV", cvv_token),
        ("xExp", exp),
    ];

    if let Some(ref inv) = invoice_number {
        params.push(("xInvoice", inv.clone()));
    }
    if let Some(ref n) = name {
        params.push(("xName", n.clone()));
    }
    if let Some(ref z) = zip {
        params.push(("xZip", z.clone()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post("https://x1.cardknox.com/gateway")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            println!("[Sola CNP] Request Error: {:?}", e);
            if e.is_timeout() {
                "Transaction timed out. Please try again.".to_string()
            } else if e.is_connect() {
                "Could not connect to payment gateway. Check your internet connection.".to_string()
            } else {
                format!("Network error: {}", e)
            }
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let http_status = response.status().as_u16();
    println!("[Sola CNP] Response Status: {} ({}ms)", http_status, duration_ms);

    let response_text = response.text().await.map_err(|e| {
        println!("[Sola CNP] Failed to read response body: {}", e);
        format!("Failed to read response: {}", e)
    })?;

    println!("[Sola CNP] Response Body: {}", response_text);

    if http_status >= 400 {
        return Err(format!("HTTP error {}: {}", http_status, response_text));
    }

    // The gateway returns form-encoded or JSON depending on request content-type
    // With form-encoded request, response is also form-encoded
    // Try JSON first, then fall back to form-encoded parsing
    let sola_response: SolaTransactionResponse = serde_json::from_str(&response_text)
        .or_else(|_| {
            // Parse form-encoded response
            let pairs: std::collections::HashMap<String, String> =
                serde_urlencoded::from_str(&response_text)
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok::<SolaTransactionResponse, String>(SolaTransactionResponse {
                x_ref_num: pairs.get("xRefNum").or_else(|| pairs.get("xRefnum")).cloned().unwrap_or_default(),
                x_result: pairs.get("xResult").cloned().unwrap_or_default(),
                x_status: pairs.get("xStatus").cloned().unwrap_or_default(),
                x_auth_code: pairs.get("xAuthCode").cloned(),
                x_masked_card_number: pairs.get("xMaskedCardNumber").cloned(),
                x_card_type: pairs.get("xCardType").cloned(),
                x_name: pairs.get("xName").cloned(),
                x_exp: pairs.get("xExp").cloned(),
                x_avs_result: pairs.get("xAvsResult").cloned(),
                x_cvv_result: pairs.get("xCvvResult").cloned(),
                x_error: pairs.get("xError").cloned(),
                x_amt: pairs.get("xAmt").cloned(),
                x_entry_method: pairs.get("xEntryMethod").cloned(),
                x_token: pairs.get("xToken").cloned(),
            })
        })
        .map_err(|e| format!("Failed to parse response: {}. Body: {}", e, response_text))?;

    println!("[Sola CNP] Parsed Response: {:?}", sola_response);

    Ok(SolaCnpTransactionResult {
        request_info,
        response: sola_response,
        raw_response: response_text,
        http_status,
        duration_ms,
    })
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

/// Save a card without processing a transaction (cc:save).
/// Returns a reusable xToken plus masked card info.
#[derive(Debug, Serialize, Deserialize)]
pub struct SolaSaveCardResult {
    pub request_info: SolaCnpRequestInfo,
    pub response: SolaTransactionResponse,
    pub raw_response: String,
    pub http_status: u16,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn sola_save_card(
    api_key: String,
    card_token: String,
    cvv_token: String,
    exp: String,
    name: Option<String>,
) -> Result<SolaSaveCardResult, String> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let version = env!("CARGO_PKG_VERSION");

    let masked_key = if api_key.len() > 10 {
        format!("{}...{}", &api_key[..5], &api_key[api_key.len() - 5..])
    } else {
        "***".to_string()
    };

    let request_info = SolaCnpRequestInfo {
        url: "https://x1.cardknox.com/gateway".to_string(),
        command: "cc:save".to_string(),
        amount: "0.00".to_string(),
        invoice: None,
        request_id: request_id.clone(),
        masked_key,
        software_name: "Vira".to_string(),
        software_version: version.to_string(),
        timestamp: {
            let duration = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
        },
    };

    // Log request (NO card data for PCI compliance)
    println!("[Sola SaveCard] Request:");
    println!("[Sola SaveCard]   URL: {}", request_info.url);
    println!("[Sola SaveCard]   xKey: {}", request_info.masked_key);
    println!("[Sola SaveCard]   xCommand: cc:save");
    // NOTE: card_token and cvv_token are NOT logged

    let start = std::time::Instant::now();

    let mut params: Vec<(&str, String)> = vec![
        ("xKey", api_key),
        ("xVersion", "5.0.0".to_string()),
        ("xSoftwareName", "Vira".to_string()),
        ("xSoftwareVersion", version.to_string()),
        ("xCommand", "cc:save".to_string()),
        ("xCardNum", card_token),
        ("xCVV", cvv_token),
        ("xExp", exp),
    ];

    if let Some(ref n) = name {
        params.push(("xName", n.clone()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post("https://x1.cardknox.com/gateway")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            println!("[Sola SaveCard] Request Error: {:?}", e);
            if e.is_timeout() {
                "Save card request timed out. Please try again.".to_string()
            } else if e.is_connect() {
                "Could not connect to payment gateway. Check your internet connection.".to_string()
            } else {
                format!("Network error: {}", e)
            }
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let http_status = response.status().as_u16();
    println!("[Sola SaveCard] Response Status: {} ({}ms)", http_status, duration_ms);

    let response_text = response.text().await.map_err(|e| {
        println!("[Sola SaveCard] Failed to read response body: {}", e);
        format!("Failed to read response: {}", e)
    })?;

    // NOTE: Do NOT log the full response body — it may contain the xToken
    println!("[Sola SaveCard] Response received ({}ms)", duration_ms);

    if http_status >= 400 {
        return Err(format!("HTTP error {}: {}", http_status, response_text));
    }

    let sola_response: SolaTransactionResponse = serde_json::from_str(&response_text)
        .or_else(|_| {
            let pairs: std::collections::HashMap<String, String> =
                serde_urlencoded::from_str(&response_text)
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok::<SolaTransactionResponse, String>(SolaTransactionResponse {
                x_ref_num: pairs.get("xRefNum").or_else(|| pairs.get("xRefnum")).cloned().unwrap_or_default(),
                x_result: pairs.get("xResult").cloned().unwrap_or_default(),
                x_status: pairs.get("xStatus").cloned().unwrap_or_default(),
                x_auth_code: pairs.get("xAuthCode").cloned(),
                x_masked_card_number: pairs.get("xMaskedCardNumber").cloned(),
                x_card_type: pairs.get("xCardType").cloned(),
                x_name: pairs.get("xName").cloned(),
                x_exp: pairs.get("xExp").cloned(),
                x_avs_result: pairs.get("xAvsResult").cloned(),
                x_cvv_result: pairs.get("xCvvResult").cloned(),
                x_error: pairs.get("xError").cloned(),
                x_amt: pairs.get("xAmt").cloned(),
                x_entry_method: pairs.get("xEntryMethod").cloned(),
                x_token: pairs.get("xToken").cloned(),
            })
        })
        .map_err(|e| format!("Failed to parse response: {}. Body: {}", e, response_text))?;

    Ok(SolaSaveCardResult {
        request_info,
        response: sola_response,
        raw_response: response_text,
        http_status,
        duration_ms,
    })
}

/// Process a transaction using a saved card token (xToken).
#[tauri::command]
pub async fn process_sola_token_transaction(
    api_key: String,
    token: String,
    amount_cents: i64,
    invoice_number: Option<String>,
    cvv: Option<String>,
) -> Result<SolaCnpTransactionResult, String> {
    let amount_dollars = format!("{:.2}", amount_cents as f64 / 100.0);
    let request_id = uuid::Uuid::new_v4().to_string();
    let version = env!("CARGO_PKG_VERSION");

    let masked_key = if api_key.len() > 10 {
        format!("{}...{}", &api_key[..5], &api_key[api_key.len() - 5..])
    } else {
        "***".to_string()
    };

    let request_info = SolaCnpRequestInfo {
        url: "https://x1.cardknox.com/gateway".to_string(),
        command: "cc:sale".to_string(),
        amount: amount_dollars.clone(),
        invoice: invoice_number.clone(),
        request_id: request_id.clone(),
        masked_key,
        software_name: "Vira".to_string(),
        software_version: version.to_string(),
        timestamp: {
            let duration = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            format!("{}.{:03}", duration.as_secs(), duration.subsec_millis())
        },
    };

    // Log request (NO token value for PCI compliance)
    println!("[Sola Token] Transaction Request:");
    println!("[Sola Token]   URL: {}", request_info.url);
    println!("[Sola Token]   xKey: {}", request_info.masked_key);
    println!("[Sola Token]   xCommand: cc:sale (token)");
    println!("[Sola Token]   xAmount: {}", amount_dollars);
    println!("[Sola Token]   xInvoice: {:?}", invoice_number);
    // NOTE: token is NOT logged

    let start = std::time::Instant::now();

    let mut params: Vec<(&str, String)> = vec![
        ("xKey", api_key),
        ("xVersion", "5.0.0".to_string()),
        ("xSoftwareName", "Vira".to_string()),
        ("xSoftwareVersion", version.to_string()),
        ("xCommand", "cc:sale".to_string()),
        ("xToken", token),
        ("xAmount", amount_dollars),
    ];

    if let Some(ref inv) = invoice_number {
        params.push(("xInvoice", inv.clone()));
    }
    if let Some(ref c) = cvv {
        params.push(("xCVV", c.clone()));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post("https://x1.cardknox.com/gateway")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            println!("[Sola Token] Request Error: {:?}", e);
            if e.is_timeout() {
                "Transaction timed out. Please try again.".to_string()
            } else if e.is_connect() {
                "Could not connect to payment gateway. Check your internet connection.".to_string()
            } else {
                format!("Network error: {}", e)
            }
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let http_status = response.status().as_u16();
    println!("[Sola Token] Response Status: {} ({}ms)", http_status, duration_ms);

    let response_text = response.text().await.map_err(|e| {
        println!("[Sola Token] Failed to read response body: {}", e);
        format!("Failed to read response: {}", e)
    })?;

    // NOTE: Do NOT log the full response body — it may contain sensitive token data
    println!("[Sola Token] Response received ({}ms, {} bytes)", duration_ms, response_text.len());

    if http_status >= 400 {
        return Err(format!("HTTP error {}: {}", http_status, response_text));
    }

    let sola_response: SolaTransactionResponse = serde_json::from_str(&response_text)
        .or_else(|_| {
            let pairs: std::collections::HashMap<String, String> =
                serde_urlencoded::from_str(&response_text)
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok::<SolaTransactionResponse, String>(SolaTransactionResponse {
                x_ref_num: pairs.get("xRefNum").or_else(|| pairs.get("xRefnum")).cloned().unwrap_or_default(),
                x_result: pairs.get("xResult").cloned().unwrap_or_default(),
                x_status: pairs.get("xStatus").cloned().unwrap_or_default(),
                x_auth_code: pairs.get("xAuthCode").cloned(),
                x_masked_card_number: pairs.get("xMaskedCardNumber").cloned(),
                x_card_type: pairs.get("xCardType").cloned(),
                x_name: pairs.get("xName").cloned(),
                x_exp: pairs.get("xExp").cloned(),
                x_avs_result: pairs.get("xAvsResult").cloned(),
                x_cvv_result: pairs.get("xCvvResult").cloned(),
                x_error: pairs.get("xError").cloned(),
                x_amt: pairs.get("xAmt").cloned(),
                x_entry_method: pairs.get("xEntryMethod").cloned(),
                x_token: pairs.get("xToken").cloned(),
            })
        })
        .map_err(|e| format!("Failed to parse response: {}. Body: {}", e, response_text))?;

    println!("[Sola Token] Parsed Response: xResult={}", sola_response.x_result);

    Ok(SolaCnpTransactionResult {
        request_info,
        response: sola_response,
        raw_response: response_text,
        http_status,
        duration_ms,
    })
}

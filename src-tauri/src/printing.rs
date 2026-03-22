use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrinterInfo {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "IsDefault")]
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct ReceiptItem {
    pub name: String,
    pub quantity: i32,
    pub unit_price: String,
    pub line_total: String,
}

#[derive(Debug, Deserialize)]
pub struct ReceiptPayment {
    pub method: String,
    pub amount: String,
    pub change: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReceiptData {
    pub store_name: String,
    pub store_address: Option<String>,
    pub store_phone: Option<String>,
    pub header: Option<String>,
    pub order_number: String,
    pub date: String,
    pub customer_name: Option<String>,
    pub items: Vec<ReceiptItem>,
    pub subtotal: String,
    pub discount: Option<String>,
    pub tax_label: String,
    pub tax: String,
    pub total: String,
    pub payments: Vec<ReceiptPayment>,
    pub footer: Option<String>,
}

// ESC/POS command constants
const ESC: u8 = 0x1B;
const GS: u8 = 0x1D;
const COLS: usize = 48; // Font A: 48 chars per line on 80mm paper

fn escpos_init() -> Vec<u8> {
    vec![ESC, 0x40] // Initialize printer
}

fn escpos_center() -> Vec<u8> {
    vec![ESC, 0x61, 0x01]
}

fn escpos_left() -> Vec<u8> {
    vec![ESC, 0x61, 0x00]
}

fn escpos_bold_on() -> Vec<u8> {
    vec![ESC, 0x45, 0x01]
}

fn escpos_bold_off() -> Vec<u8> {
    vec![ESC, 0x45, 0x00]
}

fn escpos_double_size() -> Vec<u8> {
    vec![GS, 0x21, 0x11] // Double width + double height
}

fn escpos_normal_size() -> Vec<u8> {
    vec![GS, 0x21, 0x00]
}

fn escpos_feed_and_cut(lines: u8) -> Vec<u8> {
    let mut data = vec![ESC, 0x64, lines]; // Feed n lines
    data.extend_from_slice(&[GS, 0x56, 0x42, 0x00]); // Partial cut
    data
}

fn escpos_text(text: &str) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(text.as_bytes());
    data.push(b'\n');
    data
}

fn escpos_dashes() -> Vec<u8> {
    let mut data = Vec::new();
    for _ in 0..COLS {
        data.push(b'-');
    }
    data.push(b'\n');
    data
}

/// Format a line with left and right text, padded with spaces
fn escpos_two_col(left: &str, right: &str) -> Vec<u8> {
    let left_len = left.len();
    let right_len = right.len();
    let mut line = String::with_capacity(COLS);
    line.push_str(left);
    if left_len + right_len < COLS {
        let spaces = COLS - left_len - right_len;
        for _ in 0..spaces {
            line.push(' ');
        }
    } else {
        line.push(' ');
    }
    line.push_str(right);
    let mut data = Vec::new();
    data.extend_from_slice(line.as_bytes());
    data.push(b'\n');
    data
}

fn build_receipt_escpos(receipt: &ReceiptData) -> Vec<u8> {
    let mut data = Vec::new();

    // Initialize
    data.extend(escpos_init());

    // Store header - centered, double size
    data.extend(escpos_center());
    data.extend(escpos_bold_on());
    data.extend(escpos_double_size());
    data.extend(escpos_text(&receipt.store_name));
    data.extend(escpos_normal_size());
    data.extend(escpos_bold_off());

    if let Some(ref addr) = receipt.store_address {
        if !addr.is_empty() {
            data.extend(escpos_text(addr));
        }
    }
    if let Some(ref phone) = receipt.store_phone {
        if !phone.is_empty() {
            data.extend(escpos_text(phone));
        }
    }

    // Custom header text (centered, below store info)
    if let Some(ref header) = receipt.header {
        if !header.is_empty() {
            data.extend(escpos_text("")); // blank line
            for line in header.lines() {
                data.extend(escpos_text(line));
            }
        }
    }

    // Switch to left align
    data.extend(escpos_left());
    data.extend(escpos_dashes());

    // Order info
    data.extend(escpos_two_col("Order #:", &receipt.order_number));
    data.extend(escpos_two_col("Date:", &receipt.date));
    if let Some(ref cust) = receipt.customer_name {
        if !cust.is_empty() {
            data.extend(escpos_two_col("Customer:", cust));
        }
    }

    data.extend(escpos_dashes());

    // Items
    for item in &receipt.items {
        // Line 1: Item name (left-aligned, full width)
        data.extend(escpos_text(&item.name));
        // Line 2: qty x unit_price (left) ... line_total (right)
        let detail = format!("  {} x {}", item.quantity, item.unit_price);
        data.extend(escpos_two_col(&detail, &item.line_total));
    }

    data.extend(escpos_dashes());

    // Totals
    data.extend(escpos_two_col("Subtotal:", &receipt.subtotal));
    if let Some(ref discount) = receipt.discount {
        if !discount.is_empty() {
            let disc_str = format!("-{}", discount);
            data.extend(escpos_two_col("Discount:", &disc_str));
        }
    }
    data.extend(escpos_two_col(&format!("{}:", receipt.tax_label), &receipt.tax));

    // Total in bold + double height
    data.extend(escpos_bold_on());
    data.extend(escpos_two_col("TOTAL:", &receipt.total));
    data.extend(escpos_bold_off());

    data.extend(escpos_dashes());

    // Payments
    for payment in &receipt.payments {
        data.extend(escpos_two_col(&payment.method, &payment.amount));
        if let Some(ref change) = payment.change {
            if !change.is_empty() {
                data.extend(escpos_two_col("  Change:", change));
            }
        }
        if let Some(ref reference) = payment.reference {
            if !reference.is_empty() {
                data.extend(escpos_text(&format!("  Ref: {}", reference)));
            }
        }
    }

    data.extend(escpos_dashes());

    // Footer - centered
    if let Some(ref footer) = receipt.footer {
        if !footer.is_empty() {
            data.extend(escpos_center());
            for line in footer.lines() {
                data.extend(escpos_text(line));
            }
        }
    }

    // Feed and cut
    data.extend(escpos_feed_and_cut(4));

    data
}

#[tauri::command]
pub fn print_receipt(receipt: ReceiptData, printer_name: String) -> Result<(), String> {
    println!("[Printing] Printing receipt to: {}", printer_name);

    let escpos_data = build_receipt_escpos(&receipt);

    #[cfg(target_os = "windows")]
    {
        print_raw_windows(&printer_name, &escpos_data)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        print_raw_unix(&printer_name, &escpos_data)
    }
}

#[cfg(target_os = "windows")]
fn print_raw_windows(printer_name: &str, data: &[u8]) -> Result<(), String> {
    use windows::core::{PCSTR, PSTR};
    use windows::Win32::Graphics::Printing::*;

    unsafe {
        let printer_cstr = std::ffi::CString::new(printer_name)
            .map_err(|e| format!("Invalid printer name: {}", e))?;

        let mut handle = windows::Win32::Foundation::HANDLE::default();

        // OpenPrinterA expects PCSTR
        OpenPrinterA(
            PCSTR(printer_cstr.as_ptr() as *const u8),
            &mut handle,
            None,
        )
        .map_err(|e| format!("Failed to open printer '{}': {}", printer_name, e))?;

        // DOC_INFO_1A expects PSTR (mutable pointers)
        let mut doc_name_bytes = b"Lite POS Receipt\0".to_vec();
        let mut data_type_bytes = b"RAW\0".to_vec();

        let doc_info = DOC_INFO_1A {
            pDocName: PSTR(doc_name_bytes.as_mut_ptr()),
            pOutputFile: PSTR::null(),
            pDatatype: PSTR(data_type_bytes.as_mut_ptr()),
        };

        let job_id = StartDocPrinterA(handle, 1, &doc_info);
        if job_id == 0 {
            ClosePrinter(handle).ok();
            return Err("Failed to start print job".into());
        }

        if !StartPagePrinter(handle).as_bool() {
            let _ = EndDocPrinter(handle).ok();
            let _ = ClosePrinter(handle).ok();
            return Err("Failed to start page".into());
        }

        let mut bytes_written: u32 = 0;
        let write_result = WritePrinter(
            handle,
            data.as_ptr() as *const _,
            data.len() as u32,
            &mut bytes_written,
        );

        if !write_result.as_bool() {
            let _ = EndPagePrinter(handle).ok();
            let _ = EndDocPrinter(handle).ok();
            let _ = ClosePrinter(handle).ok();
            return Err("Failed to write to printer".into());
        }

        let _ = EndPagePrinter(handle).ok();
        let _ = EndDocPrinter(handle).ok();
        let _ = ClosePrinter(handle).ok();

        println!(
            "[Printing] Successfully sent {} bytes to printer",
            bytes_written
        );
        Ok(())
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn print_raw_unix(printer_name: &str, data: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::process::Command;

    // Write raw data to temp file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("lite-pos-receipt.bin");

    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    drop(file);

    let output = Command::new("lp")
        .args(&["-d", printer_name, "-o", "raw", &temp_path.display().to_string()])
        .output()
        .map_err(|e| format!("Failed to execute lp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Print failed: {}", stderr));
    }

    Ok(())
}

// Keep the old print_html command for backwards compatibility / non-thermal printers
#[tauri::command]
pub fn print_html(html: String, printer_name: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        print_html_windows(&html, &printer_name)
    }

    #[cfg(target_os = "macos")]
    {
        print_html_unix(&html, &printer_name)
    }

    #[cfg(target_os = "linux")]
    {
        print_html_unix(&html, &printer_name)
    }
}

#[cfg(target_os = "windows")]
fn print_html_windows(_html: &str, _printer_name: &str) -> Result<(), String> {
    Err("HTML printing not supported for thermal printers. Use ESC/POS mode.".into())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn print_html_unix(html: &str, printer_name: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::Command;

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join("lite-pos-receipt.html");

    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(html.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    drop(file);

    let output = Command::new("lp")
        .args(&["-d", printer_name, &temp_path.display().to_string()])
        .output()
        .map_err(|e| format!("Failed to execute lp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Print failed: {}", stderr));
    }

    Ok(())
}

#[tauri::command]
pub fn get_system_printers() -> Result<Vec<PrinterInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        get_windows_printers()
    }

    #[cfg(target_os = "macos")]
    {
        get_macos_printers()
    }

    #[cfg(target_os = "linux")]
    {
        get_linux_printers()
    }
}

#[cfg(target_os = "windows")]
fn get_windows_printers() -> Result<Vec<PrinterInfo>, String> {
    use std::process::Command;

    // Try using WMIC first (more reliable for network printers)
    let wmic_output = Command::new("wmic")
        .args(&["printer", "get", "name,default"])
        .output();

    if let Ok(output) = wmic_output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("[Printing] WMIC output: {}", stdout);

            let mut printers = Vec::new();
            let lines: Vec<&str> = stdout.lines().collect();

            // Skip header line and parse printer names
            for line in lines.iter().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let is_default = parts[0].to_uppercase() == "TRUE";
                    let name = parts[1..].join(" ");
                    if !name.is_empty() {
                        printers.push(PrinterInfo {
                            name,
                            is_default,
                        });
                    }
                }
            }

            if !printers.is_empty() {
                println!("[Printing] Found {} printers via WMIC", printers.len());
                return Ok(printers);
            }
        }
    }

    // Fallback to PowerShell
    println!("[Printing] Trying PowerShell method...");
    let output = Command::new("powershell")
        .args(&[
            "-Command",
            "Get-Printer | Select-Object -Property Name,@{Name='IsDefault';Expression={if($_.Attributes -band 0x00000004){$true}else{$false}}} | ConvertTo-Json"
        ])
        .output()
        .map_err(|e| {
            println!("[Printing] PowerShell error: {}", e);
            format!("Failed to execute PowerShell: {}", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("[Printing] PowerShell command failed: {}", stderr);
        return Err(format!("PowerShell command failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("[Printing] PowerShell output: {}", stdout);

    if stdout.trim().is_empty() {
        println!("[Printing] No printers found");
        return Ok(vec![]);
    }

    // Handle single printer (object) vs multiple printers (array)
    let printers: Vec<PrinterInfo> = if stdout.trim().starts_with('[') {
        // Multiple printers - parse as array
        serde_json::from_str(&stdout)
            .map_err(|e| {
                println!("[Printing] JSON parse error (array): {}", e);
                format!("Failed to parse printers JSON: {}", e)
            })?
    } else if stdout.trim().starts_with('{') {
        // Single printer - parse as object and wrap in array
        let printer: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| {
                println!("[Printing] JSON parse error (object): {}", e);
                format!("Failed to parse printer JSON: {}", e)
            })?;
        vec![PrinterInfo {
            name: printer["Name"].as_str().unwrap_or("").to_string(),
            is_default: printer["IsDefault"].as_bool().unwrap_or(false),
        }]
    } else {
        println!("[Printing] Unexpected output format");
        vec![]
    };

    println!("[Printing] Found {} printers", printers.len());
    Ok(printers)
}

#[cfg(target_os = "macos")]
fn get_macos_printers() -> Result<Vec<PrinterInfo>, String> {
    use std::process::Command;

    let output = Command::new("lpstat")
        .args(&["-p", "-d"])
        .output()
        .map_err(|e| format!("Failed to execute lpstat: {}", e))?;

    if !output.status.success() {
        return Err("lpstat command failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut printers = Vec::new();
    let mut default_printer = String::new();

    for line in stdout.lines() {
        if line.starts_with("printer ") {
            let name = line
                .strip_prefix("printer ")
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                printers.push(name);
            }
        } else if line.starts_with("system default destination: ") {
            default_printer = line
                .strip_prefix("system default destination: ")
                .unwrap_or("")
                .to_string();
        }
    }

    Ok(printers
        .into_iter()
        .map(|name| PrinterInfo {
            is_default: name == default_printer,
            name,
        })
        .collect())
}

#[cfg(target_os = "linux")]
fn get_linux_printers() -> Result<Vec<PrinterInfo>, String> {
    use std::process::Command;

    let output = Command::new("lpstat")
        .args(&["-p", "-d"])
        .output()
        .map_err(|e| format!("Failed to execute lpstat: {}", e))?;

    if !output.status.success() {
        return Err("lpstat command failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut printers = Vec::new();
    let mut default_printer = String::new();

    for line in stdout.lines() {
        if line.starts_with("printer ") {
            let name = line
                .strip_prefix("printer ")
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                printers.push(name);
            }
        } else if line.starts_with("system default destination: ") {
            default_printer = line
                .strip_prefix("system default destination: ")
                .unwrap_or("")
                .to_string();
        }
    }

    Ok(printers
        .into_iter()
        .map(|name| PrinterInfo {
            is_default: name == default_printer,
            name,
        })
        .collect())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrinterInfo {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "IsDefault")]
    pub is_default: bool,
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

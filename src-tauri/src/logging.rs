use chrono::Local;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;
use tauri::Manager;

// ---------------------------------------------------------------------------
// Redaction
// ---------------------------------------------------------------------------

static CARD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(\d{13,19})\b").unwrap());

static KEY_CONTEXT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(key|token|secret|api_key|apikey|api-key|authorization|bearer)["':\s=]+([a-zA-Z0-9_\-]{20,})"#)
        .unwrap()
});

static PIN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(pin)[\"':\s=]+\S+"#).unwrap()
});

/// Redact sensitive data from a log message before writing to disk.
pub fn redact_sensitive(message: &str) -> String {
    let mut result = message.to_string();

    // Redact API keys / tokens in key-value contexts
    result = KEY_CONTEXT_RE
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}=[REDACTED]", &caps[1])
        })
        .to_string();

    // Redact card numbers (13-19 digits) — keep last 4
    result = CARD_RE
        .replace_all(&result, |caps: &regex::Captures| {
            let num = &caps[1];
            if num.len() >= 13 {
                let last4 = &num[num.len() - 4..];
                format!("****{}", last4)
            } else {
                num.to_string()
            }
        })
        .to_string();

    // Redact PIN values
    result = PIN_RE
        .replace_all(&result, |caps: &regex::Captures| {
            format!("{}=[REDACTED]", &caps[1])
        })
        .to_string();

    result
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn log_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    Ok(data_dir.join("logs"))
}

fn log_file_path(app_handle: &tauri::AppHandle, date: &str) -> Result<PathBuf, String> {
    let dir = log_dir(app_handle)?;
    Ok(dir.join(format!("vira-{}.log", date)))
}

fn level_priority(level: &str) -> u8 {
    match level.to_lowercase().as_str() {
        "error" => 0,
        "warn" => 1,
        "info" => 2,
        "debug" => 3,
        _ => 4,
    }
}

fn should_log(level: &str, threshold: &str) -> bool {
    level_priority(level) <= level_priority(threshold)
}

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50 MB

fn get_writable_log_path(app_handle: &tauri::AppHandle, date: &str) -> Result<PathBuf, String> {
    let base = log_file_path(app_handle, date)?;

    // Check base file size
    if base.exists() {
        if let Ok(meta) = fs::metadata(&base) {
            if meta.len() < MAX_FILE_SIZE {
                return Ok(base);
            }
        }
    } else {
        return Ok(base);
    }

    // Rotate: find the next available numbered file
    let dir = log_dir(app_handle)?;
    for i in 1..100 {
        let rotated = dir.join(format!("vira-{}.{}.log", date, i));
        if !rotated.exists() {
            return Ok(rotated);
        }
        if let Ok(meta) = fs::metadata(&rotated) {
            if meta.len() < MAX_FILE_SIZE {
                return Ok(rotated);
            }
        }
    }

    Err("Too many log rotation files for today".to_string())
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub category: String,
    pub message: String,
    pub raw: String,
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // Format: [YYYY-MM-DD HH:MM:SS.mmm] [LEVEL] [SOURCE] [CATEGORY] MESSAGE
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }

    let mut parts = Vec::new();
    let mut rest = line;

    // Extract up to 4 bracketed sections
    for _ in 0..4 {
        if !rest.starts_with('[') {
            break;
        }
        if let Some(end) = rest.find(']') {
            parts.push(&rest[1..end]);
            rest = rest[end + 1..].trim_start();
        } else {
            break;
        }
    }

    if parts.len() < 4 {
        return None;
    }

    Some(LogEntry {
        timestamp: parts[0].to_string(),
        level: parts[1].to_string(),
        source: parts[2].to_string(),
        category: parts[3].to_string(),
        message: rest.to_string(),
        raw: line.to_string(),
    })
}

#[tauri::command]
pub fn log_event(
    app_handle: tauri::AppHandle,
    level: String,
    source: String,
    category: String,
    message: String,
    log_level_threshold: Option<String>,
    enabled: Option<bool>,
) -> Result<(), String> {
    // Check if logging is enabled (passed from frontend, defaults to true)
    if enabled == Some(false) {
        return Ok(());
    }

    // Check level threshold (passed from frontend, defaults to "info")
    let threshold = log_level_threshold.unwrap_or_else(|| "info".to_string());
    if !should_log(&level, &threshold) {
        return Ok(());
    }

    // Redact sensitive data
    let safe_message = redact_sensitive(&message);

    // Format the log line
    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let date = now.format("%Y-%m-%d").to_string();
    let level_upper = level.to_uppercase();

    let line = format!(
        "[{}] [{}] [{}] [{}] {}\n",
        timestamp, level_upper, source, category, safe_message
    );

    // Ensure log directory exists
    let dir = log_dir(&app_handle)?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create log dir: {}", e))?;
    }

    // Get writable path (handles rotation)
    let path = get_writable_log_path(&app_handle, &date)?;

    // Append to file
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    file.write_all(line.as_bytes())
        .map_err(|e| format!("Failed to write log entry: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_log_entries(
    app_handle: tauri::AppHandle,
    date: Option<String>,
    level: Option<String>,
    category: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    let target_date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let dir = log_dir(&app_handle)?;

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    let max = limit.unwrap_or(500);

    // Collect all log files for the target date (including rotated ones)
    let mut files: Vec<PathBuf> = Vec::new();
    let base = dir.join(format!("vira-{}.log", target_date));
    if base.exists() {
        files.push(base);
    }
    for i in 1..100 {
        let rotated = dir.join(format!("vira-{}.{}.log", target_date, i));
        if rotated.exists() {
            files.push(rotated);
        } else {
            break;
        }
    }

    for file_path in files {
        let content =
            fs::read_to_string(&file_path).map_err(|e| format!("Failed to read log file: {}", e))?;

        for line in content.lines() {
            if let Some(entry) = parse_log_line(line) {
                // Apply level filter
                if let Some(ref lvl) = level {
                    if entry.level.to_lowercase() != lvl.to_lowercase() {
                        continue;
                    }
                }
                // Apply category filter
                if let Some(ref cat) = category {
                    if !cat.is_empty() && entry.category.to_lowercase() != cat.to_lowercase() {
                        continue;
                    }
                }
                entries.push(entry);
                if entries.len() >= max {
                    return Ok(entries);
                }
            }
        }
    }

    Ok(entries)
}

#[tauri::command]
pub fn get_log_dates(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let dir = log_dir(&app_handle)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut dates: Vec<String> = Vec::new();
    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read log directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Match vira-YYYY-MM-DD.log or vira-YYYY-MM-DD.N.log
        if name.starts_with("vira-") && name.ends_with(".log") {
            // Extract date part
            let without_prefix = &name[5..]; // remove "vira-"
            if let Some(dot_pos) = without_prefix.find('.') {
                let date = &without_prefix[..dot_pos];
                // Validate date format (10 chars: YYYY-MM-DD)
                if date.len() == 10 && !dates.contains(&date.to_string()) {
                    dates.push(date.to_string());
                }
            }
        }
    }

    dates.sort();
    dates.reverse(); // Most recent first
    Ok(dates)
}

#[tauri::command]
pub fn export_log(
    app_handle: tauri::AppHandle,
    date: Option<String>,
) -> Result<String, String> {
    let target_date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let dir = log_dir(&app_handle)?;

    if !dir.exists() {
        return Ok(String::new());
    }

    let mut content = String::new();

    // Read base file and all rotated files
    let base = dir.join(format!("vira-{}.log", target_date));
    if base.exists() {
        content += &fs::read_to_string(&base)
            .map_err(|e| format!("Failed to read log file: {}", e))?;
    }
    for i in 1..100 {
        let rotated = dir.join(format!("vira-{}.{}.log", target_date, i));
        if rotated.exists() {
            content += &fs::read_to_string(&rotated)
                .map_err(|e| format!("Failed to read rotated log file: {}", e))?;
        } else {
            break;
        }
    }

    Ok(content)
}

#[tauri::command]
pub fn purge_old_logs(
    app_handle: tauri::AppHandle,
    retention_days: Option<i64>,
) -> Result<u64, String> {
    let days = retention_days.unwrap_or(30);
    let dir = log_dir(&app_handle)?;

    if !dir.exists() {
        return Ok(0);
    }

    let delete_all = days == 0;
    let cutoff = Local::now() - chrono::Duration::days(days);
    let cutoff_date = cutoff.format("%Y-%m-%d").to_string();
    let mut freed: u64 = 0;

    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read log directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with("vira-") && name.ends_with(".log") {
            let without_prefix = &name[5..];
            if let Some(dot_pos) = without_prefix.find('.') {
                let date = &without_prefix[..dot_pos];
                if date.len() == 10 && (delete_all || date < cutoff_date.as_str()) {
                    if let Ok(meta) = fs::metadata(entry.path()) {
                        freed += meta.len();
                    }
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    Ok(freed)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_card_numbers() {
        assert_eq!(
            redact_sensitive("Card: 4111111111111111"),
            "Card: ****1111"
        );
        assert_eq!(
            redact_sensitive("Number 5500000000000004 used"),
            "Number ****0004 used"
        );
        // Short numbers should not be redacted
        assert_eq!(redact_sensitive("Order 12345"), "Order 12345");
    }

    #[test]
    fn test_redact_api_keys() {
        assert_eq!(
            redact_sensitive("api_key=abcdefghijklmnopqrst1234"),
            "api_key=[REDACTED]"
        );
        assert_eq!(
            redact_sensitive("token: ABCDEFGHIJKLMNOPQRSTUVWX"),
            "token=[REDACTED]"
        );
        assert_eq!(
            redact_sensitive("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9"),
            "Authorization: Bearer=[REDACTED]"
        );
    }

    #[test]
    fn test_redact_pins() {
        assert_eq!(
            redact_sensitive("pin=1234"),
            "pin=[REDACTED]"
        );
        assert_eq!(
            redact_sensitive("\"pin\": \"5678\""),
            "\"pin=[REDACTED]"
        );
    }

    #[test]
    fn test_no_false_positives() {
        // Normal messages should pass through
        assert_eq!(
            redact_sensitive("Order #12345 completed for $50.00"),
            "Order #12345 completed for $50.00"
        );
        assert_eq!(
            redact_sensitive("Customer sync completed: 45 synced"),
            "Customer sync completed: 45 synced"
        );
    }

    #[test]
    fn test_parse_log_line() {
        let line = "[2026-03-21 14:23:45.123] [INFO] [backend] [auth] User logged in";
        let entry = parse_log_line(line).unwrap();
        assert_eq!(entry.timestamp, "2026-03-21 14:23:45.123");
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.source, "backend");
        assert_eq!(entry.category, "auth");
        assert_eq!(entry.message, "User logged in");
    }

    #[test]
    fn test_level_filtering() {
        assert!(should_log("error", "info"));
        assert!(should_log("warn", "info"));
        assert!(should_log("info", "info"));
        assert!(!should_log("debug", "info"));
        assert!(should_log("error", "error"));
        assert!(!should_log("warn", "error"));
    }
}

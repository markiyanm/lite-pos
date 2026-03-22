# Architecture: Application Logging System with Admin Viewer

## Overview

File-based logging system with a Rust backend for log writing/reading and a SvelteKit frontend for viewing/exporting. Logs are stored as plain text files in the Tauri app data directory, one file per day.

## System Components

### 1. Rust Module: `src-tauri/src/logging.rs`

**Responsibilities:**
- Write log entries to daily log files (append-only)
- Read/filter log entries for the admin viewer
- List available log dates
- Export log file content as string
- Purge old log files based on retention setting
- Redact sensitive data (API keys, card numbers, PINs)

**Key Functions:**
- `log_event(level, source, category, message)` - Tauri command; validates level against settings, applies redaction, appends to daily file
- `get_log_entries(date?, level?, category?, limit?)` - Tauri command; reads/filters log entries from file
- `get_log_dates()` - Tauri command; lists dates for which log files exist
- `export_log(date?)` - Tauri command; returns full file content as string
- `purge_old_logs()` - Tauri command; deletes files older than retention setting
- `redact_sensitive(message: &str) -> String` - Internal; regex-based redaction

**Log File Location:** `{app_data_dir}/logs/vira-YYYY-MM-DD.log`

**Log Format:**
```
[YYYY-MM-DD HH:MM:SS.mmm] [LEVEL] [SOURCE] [CATEGORY] MESSAGE
```

**File Size Cap:** 50MB per file. When exceeded, rotate to `.1.log`, `.2.log`, etc.

**Redaction Rules:**
1. Card numbers: 13-19 consecutive digits → `****` + last 4
2. Strings matching API key patterns (20+ alphanumeric chars near key/token/secret context words) → `[REDACTED]`
3. PIN values in `pin=` or `"pin":` contexts → `[REDACTED]`

### 2. Database Migration: `010_logging.sql`

Insert default settings into the existing `settings` table:
- `enable_logging` (boolean, default: "true", group: "diagnostics")
- `log_retention_days` (integer, default: "30", group: "diagnostics")
- `log_level` (string, default: "info", group: "diagnostics")

No new tables needed — logs are file-based, not SQLite-based.

### 3. Frontend Commands: `src/lib/commands/logging.ts`

Thin wrappers around Tauri `invoke()` calls:
- `logEvent(level, category, message)` - always passes `source: 'frontend'`
- `getLogEntries(filters)` - returns parsed log entries
- `getLogDates()` - returns string array of dates
- `exportLog(date)` - returns log text

### 4. Logger Utility: `src/lib/utils/logger.ts`

Convenience wrapper with level methods:
```typescript
log.info('category', 'message')
log.warn('category', 'message')
log.error('category', 'message')
log.debug('category', 'message')
```

Each method calls `logEvent()` and catches errors silently to never crash the app.

### 5. Settings UI: Diagnostics Tab

New tab in Settings page (admin-only) with:
- Enable/disable logging toggle
- Log level selector (error, warn, info, debug)
- Retention days input (7-90 range)
- "View Logs" button → opens LogViewerDialog
- "Clear Old Logs" button with confirmation

### 6. LogViewerDialog Component

`src/lib/components/settings/LogViewerDialog.svelte`

Dialog with:
- Date picker (select from available dates)
- Level filter checkboxes
- Category filter dropdown
- Monospace scrollable log display
- Color-coded levels
- Export button (triggers Blob download since no dialog plugin installed)
- Auto-scroll to bottom

### 7. Integration Points

Add `log.*()` calls to:
- `src/lib/commands/auth.ts` - login success/failure, logout
- `src/lib/commands/orders.ts` - order created, completed, voided
- `src/lib/commands/payments.ts` - payment processed
- `src/routes/settings/+page.svelte` - setting changes
- `src/routes/+layout.svelte` - app startup

## Data Flow

```
Frontend log.info() → logEvent() → invoke("log_event") → Rust:
  1. Check if logging enabled (read setting from DB)
  2. Check if level meets threshold
  3. Apply redact_sensitive() to message
  4. Format line with timestamp
  5. Append to daily log file (buffered I/O)
```

## Error Handling Strategy

- All logging calls wrapped in try/catch that silently swallow errors
- Rust log_event returns Ok(()) even if file write fails (logs to stderr instead)
- Frontend logger utility never throws
- Missing log directory is auto-created on first write

## Dependencies

- `chrono` crate needed for timestamp formatting in Rust
- `regex` crate for redaction patterns
- `tauri-plugin-fs` for file system access from Rust (already available via std::fs)
- No new Tauri plugins required for frontend (export uses Blob download)

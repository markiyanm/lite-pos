# Requirements: Application Logging System with Admin Viewer

## Problem Statement

Vira currently has no persistent logging. Errors are printed to stdout (Rust) or console.log (frontend) and are lost when the app closes. When merchants report issues, there is no way for tech support to diagnose what happened. This is especially critical as we add gateway integrations (customer sync, iFields, tokenization) where failures need to be traceable.

## User Stories

- As a tech support agent, I want to see what happened on a merchant's POS so I can diagnose issues remotely.
- As an admin, I want to view recent logs in the app so I can self-diagnose simple problems.
- As an admin, I want to export logs as a text file to email to tech support.
- As a merchant, I want logging to not slow down my POS or fill up my disk.
- As a merchant, I want to be confident that my API keys and customer card numbers are never in the logs.

## Functional Requirements

### Settings

1. **FR-1:** Add a boolean setting `enable_logging` in a new "Diagnostics" subsection of Settings. Default: `true`.
2. **FR-2:** Add a setting `log_retention_days` with default `30`. Range: 7-90.
3. **FR-3:** Add a setting `log_level` with options: `error`, `warn`, `info`, `debug`. Default: `info`.

### Log Storage (Rust Backend)

4. **FR-4:** Create a new Tauri module `src-tauri/src/logging.rs` that manages log files.
5. **FR-5:** Log files are stored in the Tauri app data directory under `logs/` subdirectory.
6. **FR-6:** One log file per day, named `vira-YYYY-MM-DD.log`. Append-only.
7. **FR-7:** Log format per line:
   ```
   [YYYY-MM-DD HH:MM:SS.mmm] [LEVEL] [SOURCE] [CATEGORY] MESSAGE
   ```
   Example:
   ```
   [2026-03-21 14:23:45.123] [INFO] [backend] [gateway-sync] Customer sync completed: 45 synced, 2 errors
   [2026-03-21 14:23:46.001] [ERROR] [backend] [gateway-sync] Failed to sync customer #23: HTTP 500
   ```
8. **FR-8:** Tauri commands:
   - `log_event(level, source, category, message)` — append a log entry
   - `get_log_entries(date?, level?, category?, limit?)` — read log entries with filters
   - `get_log_dates()` — list available log file dates
   - `export_log(date?)` — return full log file content as a string (for download)
   - `purge_old_logs()` — delete log files older than `log_retention_days`

### Sensitive Data Redaction

9. **FR-9:** Before writing any log entry, apply redaction rules:
   - API keys: replace any string matching `[a-zA-Z0-9]{20,}` that appears in a key/token context with `[REDACTED]`
   - Card numbers: replace any 13-19 digit sequence with `****XXXX` (last 4 only)
   - PINs: never log PIN values
   - The redaction function must be called on every log message before writing to disk.
10. **FR-10:** Create a `redact_sensitive(message: &str) -> String` function in Rust that applies all redaction rules.

### Auto-Purge

11. **FR-11:** On app startup, call `purge_old_logs()` to delete files older than `log_retention_days`.
12. **FR-12:** Log files must never exceed 50MB per file. If a day's log exceeds 50MB, rotate to `vira-YYYY-MM-DD.1.log`, etc.

### Frontend Logging Bridge

13. **FR-13:** Create `src/lib/commands/logging.ts` with:
    - `logEvent(level, category, message)` — invokes Tauri command with `source: 'frontend'`
    - `getLogEntries(filters)` — retrieves filtered log entries
    - `getLogDates()` — retrieves available dates
    - `exportLog(date)` — retrieves full log text
14. **FR-14:** Create a `log` utility in `src/lib/utils/logger.ts` for convenient logging:
    ```typescript
    log.info('category', 'message')
    log.warn('category', 'message')
    log.error('category', 'message')
    log.debug('category', 'message')
    ```

### What to Log (Integration Points)

15. **FR-15:** Log these events at minimum:
    - `[auth]` — login success/failure, logout (never log PIN)
    - `[order]` — order created, completed, voided, refunded (log order number, amount)
    - `[payment]` — payment processed, failed (log method, amount, last 4 digits only)
    - `[gateway-sync]` — sync started, completed, per-customer errors
    - `[settings]` — setting changed (log key and new value, except for encrypted settings)
    - `[startup]` — app started, version number, platform info
    - `[error]` — any unhandled errors

### Admin UI

16. **FR-16:** Add a "View Logs" button in the Diagnostics settings section. Only visible to admin users.
17. **FR-17:** The log viewer is a dialog/modal with:
    - Date picker to select which day's logs to view
    - Level filter (checkboxes: error, warn, info, debug)
    - Category filter dropdown
    - Scrollable log display with monospace font
    - Color-coded levels (red=error, orange=warn, default=info, gray=debug)
    - Auto-scroll to bottom (latest entries)
18. **FR-18:** "Export" button that triggers a file save dialog via Tauri's `dialog` plugin, saving the current day's log as `vira-logs-YYYY-MM-DD.txt`.
19. **FR-19:** "Clear Old Logs" button that manually triggers `purge_old_logs()` and shows how much space was freed.

## Non-Functional Requirements

- **Performance:** Logging must add < 1ms overhead per call. Use buffered file I/O.
- **Disk Usage:** 30 days of logs for a typical store should be < 100MB. 50MB per-file cap with rotation.
- **Security:** No API keys, full card numbers, PINs, or passwords may appear in logs under any circumstance.
- **Reliability:** Logging failures must never crash the app. If the log file can't be written, silently skip.

## Acceptance Criteria

- [ ] Setting to enable/disable logging exists and defaults to on
- [ ] Log files are created in app data `logs/` directory, one per day
- [ ] Log entries follow the specified format with timestamp, level, source, category
- [ ] API keys in log messages are redacted to `[REDACTED]`
- [ ] Card numbers in log messages are redacted to `****XXXX`
- [ ] Logs older than retention period are auto-purged on startup
- [ ] Admin can view logs in the app via the Diagnostics section
- [ ] Admin can filter logs by date, level, and category
- [ ] Admin can export logs as a plain text file
- [ ] Log level setting controls which levels are written (e.g., `warn` writes warn + error only)
- [ ] Frontend can log events via the `log` utility
- [ ] Logging failure does not crash the app

## Out of Scope

- Remote/cloud log shipping (future feature)
- Real-time log streaming in the UI (poll on demand is fine)
- Log search/regex filtering (simple level+category filter is enough)
- Log rotation within a single day beyond 50MB cap

## Dependencies

- None (foundational feature — other features will use this)

## Open Questions

- None. Requirements are clear and self-contained.

# Implementation Plan: Application Logging System with Admin Viewer

## Step 1: Database Migration

Create `src-tauri/migrations/010_logging.sql`:
- INSERT three settings: `enable_logging`, `log_retention_days`, `log_level`
- Register migration in `src-tauri/src/lib.rs`

## Step 2: Rust Logging Module

Create `src-tauri/src/logging.rs`:
1. Add `chrono` and `regex` to Cargo.toml
2. Implement `redact_sensitive(message: &str) -> String`
3. Implement `get_log_dir(app_handle) -> PathBuf`
4. Implement `get_log_file_path(app_handle, date) -> PathBuf`
5. Implement `should_log(level, threshold) -> bool` level hierarchy
6. Implement Tauri commands:
   - `log_event` - validate, redact, append to file
   - `get_log_entries` - read file, parse lines, filter
   - `get_log_dates` - list log files, extract dates
   - `export_log` - read full file content
   - `purge_old_logs` - delete old files, return freed bytes

## Step 3: Register Rust Commands

Update `src-tauri/src/lib.rs`:
- Add `mod logging;`
- Add all 5 commands to `invoke_handler`
- Call `purge_old_logs` in `.setup()` callback

## Step 4: Frontend Commands

Create `src/lib/commands/logging.ts`:
- `logEvent(level, category, message)` wrapper
- `getLogEntries(filters?)` wrapper
- `getLogDates()` wrapper
- `exportLog(date?)` wrapper
- `purgeOldLogs()` wrapper

## Step 5: Logger Utility

Create `src/lib/utils/logger.ts`:
- Export `log` object with `.info()`, `.warn()`, `.error()`, `.debug()` methods
- Each method silently catches errors
- All pass `source: 'frontend'` implicitly

## Step 6: LogViewerDialog Component

Create `src/lib/components/settings/LogViewerDialog.svelte`:
- Props: `open`, `onClose`
- State: selected date, level filters, category filter, entries array, loading
- Load dates on open, load entries when filters change
- Display entries in monospace scrollable div with color coding
- Export button: create Blob, trigger download via anchor tag
- Clear old logs button with toast feedback

## Step 7: Diagnostics Tab in Settings

Update `src/routes/settings/+page.svelte`:
- Add state variables for diagnostics settings
- Add "diagnostics" TabsTrigger (admin-only)
- Add TabsContent with enable toggle, level select, retention input
- Add View Logs button → open LogViewerDialog
- Include diagnostics settings in saveSettings()

## Step 8: Integration Points

Add logging calls to:
- `src/lib/commands/auth.ts` - after login/logout
- `src/lib/commands/orders.ts` - after createOrder, completeOrder, voidOrder
- `src/lib/commands/payments.ts` - after addPayment
- `src/routes/settings/+page.svelte` - in saveSettings()
- `src/routes/+layout.svelte` - on mount for startup log

## Step 9: Testing

- Run `npm run check` for type errors
- Run `cargo check` in src-tauri/ for Rust compilation
- Verify redaction edge cases in code review

# Design: Application Logging System with Admin Viewer

## Diagnostics Settings Section

Location: New tab "Diagnostics" in Settings page, visible only to admin users.

### Layout

```
Card: Diagnostics
  CardHeader: "Diagnostics" / "Configure application logging and view system logs."
  CardContent:
    - SettingToggle: "Enable Logging" / "Record application events for troubleshooting"
    - Log Level: Select dropdown [Error, Warning, Info, Debug]
    - Retention: Number input "Log Retention (days)" with range 7-90
    - Separator
    - Action Buttons row:
      - [View Logs] button (primary) → opens LogViewerDialog
      - [Clear Old Logs] button (outline/destructive) → purge with confirmation toast
```

## LogViewerDialog

Full-screen-ish dialog (max-w-4xl) with structured layout.

### Header
- Title: "Application Logs"
- Description: "View and export application log entries"

### Filter Bar (horizontal row, flex-wrap)
- Date selector: native HTML date input or shadcn Select populated with available dates
- Level filters: 4 checkboxes (Error, Warn, Info, Debug) — all checked by default
- Category filter: Select dropdown with "All Categories" default
- Entry limit: small input, default 500

### Log Display Area
- Container: `max-h-[60vh] overflow-y-auto` with monospace font
- Each log line rendered as a div
- Color coding via Tailwind:
  - ERROR: `text-red-500`
  - WARN: `text-orange-500`
  - INFO: `text-foreground` (default)
  - DEBUG: `text-muted-foreground`
- Empty state: "No log entries found" centered message
- Loading state: Loader2 spinner

### Footer
- Left: entry count ("Showing X entries")
- Right:
  - [Export] button → downloads current day's log as .txt file
  - [Close] button

### Export Behavior
Since no Tauri dialog plugin is installed, export creates a Blob and triggers download via a temporary anchor element. File named `vira-logs-YYYY-MM-DD.txt`.

## Color Scheme

Follows existing app theme. Log viewer uses:
- Monospace font: `font-mono text-xs` for log entries
- Background: card background (inherits dark/light mode)
- Borders: standard `border` class
- Buttons: shadcn Button variants (default, outline, destructive)

## Responsive Behavior

- Filter bar wraps on smaller screens
- Log display takes full available width
- Dialog is scrollable if content exceeds viewport

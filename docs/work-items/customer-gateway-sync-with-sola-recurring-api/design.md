# Design: Customer Gateway Sync UI

## Customer List (customers/+page.svelte)

### Sync Status Icons
Each customer row gets a small icon after the name column indicating gateway sync status:
- **synced**: `CloudCheck` icon (Lucide), green (`text-emerald-500`), 16px
- **pending**: `CloudUpload` icon (Lucide), blue (`text-blue-500`), 16px, with `animate-pulse`
- **error**: `CloudAlert` icon (Lucide), orange (`text-orange-500`), 16px
- **archived/orphaned**: `CloudOff` icon (Lucide), muted (`text-muted-foreground`), 16px
- **unsynced**: No icon (clean default state)

Placement: New "Sync" column between "City" and "Actions" columns, narrow width, center-aligned.

## Customer Detail (customers/[id]/+page.svelte)

### Sync Status Badge
In the sidebar, below the Notes card, add a "Gateway Sync" card (only shown for existing customers when sync is enabled):
- Badge showing current status with color matching the list icons
- "Last synced: [relative time]" text below the badge
- If error: show error status in destructive badge
- If unsynced: show "Not yet synced" in secondary badge

## Settings - Payments Tab

### Customer Sync Section
Added inside the Payments tab, after the Sola Gateway card, only visible when API key is configured:
- Card titled "Customer Sync"
- Toggle: "Auto-sync customers" (gateway_customer_sync_enabled)
- Input: "Sync interval (minutes)" with min=5 max=1440 (gateway_sync_interval_minutes)
- "Sync Now" button:
  - Default: outline variant, with `RefreshCw` icon
  - Syncing: disabled, `Loader2` with spin animation, text "Syncing..."
  - After sync: brief success/error toast via svelte-sonner
- Small helper text showing last sync result if available

## Sync Progress (Non-blocking)
- No modal or blocking UI during sync
- Toast notification only on completion IF errors > 0
- Events listened via Tauri `listen()` in +layout.svelte

## Color Tokens
All sync status colors use Tailwind utility classes:
- Green: `text-emerald-500`
- Blue: `text-blue-500`
- Orange: `text-orange-500`
- Muted: `text-muted-foreground`

# Architecture: Customer Gateway Sync

## Overview

Bidirectional sync engine between local SQLite customer records and Sola Recurring API v2 (Cardknox). Runs as a background Tauri task with event-driven frontend updates.

## Component Diagram

```
Frontend (SvelteKit)                          Backend (Rust/Tauri)
+-----------------------+                     +------------------------+
| customer-sync.ts      | --invoke-->         | customer_sync.rs       |
|  - triggerSync()      |                     |  - sync_customers_with |
|  - listenSyncEvents() |                     |    _gateway()          |
+-----------------------+                     |  - bidirectional match |
| customers/+page.svelte|                     |  - field mapping       |
|  - sync status icons  | <--events--         +----------+-------------+
| customers/[id]/       |                                |
|  - sync status badge  |                     +----------v-------------+
| settings/+page.svelte |                     | recurring_api.rs       |
|  - "Sync Now" button  |                     |  - list_customers()    |
+-----------------------+                     |  - create_customer()   |
                                              |  - update_customer()   |
                                              |  - delete_customer()   |
                                              |  - get_customer()      |
                                              +----------+-------------+
                                                         |
                                              +----------v-------------+
                                              | Sola Recurring API v2  |
                                              | api.cardknox.com/v2    |
                                              +------------------------+
```

## Data Flow

1. **Startup**: App loads -> 5s delay -> auto-sync if enabled + API key configured
2. **Periodic**: tokio interval timer at configured minutes
3. **Manual**: User clicks "Sync Now" in Settings
4. **Sync cycle**: List gateway customers -> match by gateway_customer_id -> push/pull/create

## Database Changes (Migration 010)

### customers table additions
- `gateway_customer_id` TEXT - Sola's CustomerId
- `gateway_sync_status` TEXT DEFAULT 'unsynced' - enum: unsynced|synced|pending|error|archived|orphaned
- `gateway_synced_at` TEXT - last successful sync timestamp
- `gateway_revision` INTEGER - Sola optimistic locking revision

### New: sync_log table
- Audit trail for all sync operations
- Direction (push/pull), action (create/update/delete), status, error message

## Sync Algorithm

1. Fetch all gateway customers (paginated, 100/page)
2. Fetch all local customers (not deleted)
3. Build lookup maps: local by gateway_customer_id, gateway by CustomerNumber (our uuid)
4. For each gateway customer:
   - If matched locally: compare revisions, pull if gateway newer
   - If unmatched: create local record with gateway data
5. For each local customer without gateway_customer_id:
   - Push to gateway via CreateCustomer
   - Store returned CustomerId and Revision
6. For matched customers where local updated_at > gateway_synced_at:
   - Push update via UpdateCustomer (all fields required)
7. Handle deletes per FR-9/10/11

## Security

- API key stays in Rust backend, never sent to frontend
- API key masked in all log output
- Frontend receives only sync status events, no credentials

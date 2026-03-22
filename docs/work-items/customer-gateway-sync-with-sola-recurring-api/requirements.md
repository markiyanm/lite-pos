# Requirements: Customer Gateway Sync with Sola Recurring API

## Problem Statement

Vira currently stores customers only in the local SQLite database. Merchants using Sola Payments for recurring billing or card-on-file need their customer records synchronized with the Sola recurring API (v2) so that payment methods, schedules, and customer data remain consistent across both systems. Without sync, merchants must manually maintain customer records in two places, leading to data drift, failed payments, and wasted time.

## User Stories

- As a merchant, I want my local customers to automatically sync with Sola so I don't have to enter customer data twice.
- As a cashier, I want to see which customers are synced with the gateway so I know their payment methods are up to date.
- As a merchant, I want sync to happen in the background so it doesn't slow down my sales workflow.
- As a merchant, I want the POS to keep working with local data even when the gateway is unreachable.
- As a merchant, I want to control whether gateway sync is on or off from Settings.

## Functional Requirements

### Settings

1. **FR-1:** Add a boolean setting `gateway_customer_sync_enabled` in the Payments tab of Settings. Default: `true` (on).
2. **FR-2:** Add a numeric setting `gateway_sync_interval_minutes` with default `15`. Range: 5-1440.
3. **FR-3:** Sync settings must only be visible when `sola_gateway_api_key` is configured.

### Database Schema (Migration 010)

4. **FR-4:** Add columns to the `customers` table:
   - `gateway_customer_id` TEXT (nullable) — the Sola CustomerId
   - `gateway_sync_status` TEXT DEFAULT 'unsynced' — enum: 'unsynced', 'synced', 'pending', 'error'
   - `gateway_synced_at` TEXT (nullable) — ISO 8601 timestamp of last successful sync
   - `gateway_revision` INTEGER (nullable) — Sola revision number for optimistic locking
5. **FR-5:** Add a `sync_log` table for tracking sync operations:
   - `id`, `entity_type` ('customer'), `entity_id`, `direction` ('push'|'pull'), `action` ('create'|'update'|'delete'), `status` ('success'|'error'), `error_message`, `created_at`

### Sync Engine (Rust Backend)

6. **FR-6:** Create a new Tauri command `sync_customers_with_gateway(apiKey)` that:
   a. Calls Sola `ListCustomers` (paginated, up to 100 per page) to get all gateway customers
   b. Matches gateway customers to local customers by `gateway_customer_id`
   c. For unmatched gateway customers: creates local customer record with gateway data
   d. For unmatched local customers (no `gateway_customer_id`): calls Sola `CreateCustomer` to push to gateway, stores returned `CustomerId` and `Revision`
   e. For matched customers where local `updated_at` > `gateway_synced_at`: calls Sola `UpdateCustomer` (must send ALL fields per API requirement, not just changed ones)
   f. For matched customers where gateway revision > local `gateway_revision`: pulls gateway data into local
   g. Updates `gateway_sync_status`, `gateway_synced_at`, `gateway_revision` on success
   h. Sets `gateway_sync_status = 'error'` on failure per-customer

7. **FR-7:** Implement field mapping between local schema and Sola API:
   | Local Field | Sola Field |
   |---|---|
   | first_name | BillFirstName |
   | last_name | BillLastName |
   | email | Email |
   | phone | BillPhone |
   | billing_address_line1 | BillStreet |
   | billing_address_city | BillCity |
   | billing_address_state | BillState |
   | billing_address_zip | BillZip |
   | billing_address_country | BillCountry |
   | uuid | CustomerNumber (for correlation) |

8. **FR-8:** API calls must include `SoftwareName: "Vira"` and `SoftwareVersion: <cargo_pkg_version>`.

### Delete Handling

9. **FR-9:** When a local customer with orders is "deleted" (soft-deleted via `deleted_at`), do NOT delete from gateway. Set `gateway_sync_status = 'archived'`.
10. **FR-10:** When a local customer with zero orders is deleted, call Sola `DeleteCustomer` if they have a `gateway_customer_id`. If the gateway delete fails (e.g., active schedules), log the error but still soft-delete locally.
11. **FR-11:** When gateway returns a deleted customer during sync, soft-delete locally ONLY if the customer has zero orders. Otherwise, mark as `gateway_sync_status = 'orphaned'` and log a warning.

### Background Execution

12. **FR-12:** On app startup, if `gateway_customer_sync_enabled` is true and API key is configured, trigger a sync after a 5-second delay (to not block the UI).
13. **FR-13:** After startup sync, schedule periodic syncs at the configured interval using a Rust background task (tokio interval timer on the Tauri async runtime).
14. **FR-14:** Sync must run entirely in the background. The frontend receives progress via Tauri events:
    - `gateway-sync-started` — with `{ entity: 'customers' }`
    - `gateway-sync-progress` — with `{ entity: 'customers', processed: N, total: M }`
    - `gateway-sync-completed` — with `{ entity: 'customers', created: N, updated: N, errors: N }`
    - `gateway-sync-error` — with `{ entity: 'customers', message: string }`

### Graceful Failure

15. **FR-15:** If the Sola API returns HTTP 401/403 (bad API key), emit `gateway-sync-error` with a clear message and disable auto-sync until the next app restart or manual settings change.
16. **FR-16:** If the Sola API is unreachable (network error, timeout), retry once after 30 seconds, then emit `gateway-sync-error` and wait for the next scheduled interval.
17. **FR-17:** Individual customer sync failures must NOT abort the entire batch. Process all customers and report per-customer errors.

### UI

18. **FR-18:** Show a sync status icon next to each customer in the customer list:
    - Cloud check icon (green) for `synced`
    - Cloud upload icon (blue, animated) for `pending`
    - Cloud alert icon (orange) for `error`
    - No icon for `unsynced`
19. **FR-19:** In the customer detail view, show sync status as a badge with the last sync timestamp.
20. **FR-20:** Add a "Sync Now" button in the Payments settings section that manually triggers `sync_customers_with_gateway`.
21. **FR-21:** Show a subtle toast notification when background sync completes (only if errors > 0).

## Non-Functional Requirements

- **Performance:** Sync must process 500+ customers within 2 minutes. Use pagination (100 per page).
- **Resilience:** App must remain fully functional with local data when gateway is unavailable.
- **Security:** API key is never logged or sent to frontend. All gateway HTTP calls happen in Rust backend.
- **Concurrency:** Only one sync operation at a time. If a sync is already running, skip the interval trigger.

## Acceptance Criteria

- [ ] Setting to enable/disable gateway sync exists and defaults to on
- [ ] On startup with sync enabled and API key configured, customers sync within 10 seconds
- [ ] Periodic sync runs at configured interval
- [ ] New local customer is pushed to gateway and receives a `gateway_customer_id`
- [ ] Gateway customer not in local DB is pulled and created locally
- [ ] Updated local customer is pushed to gateway with correct revision
- [ ] Customer with orders is NOT deleted from gateway on local soft-delete
- [ ] Customer with 0 orders IS deleted from gateway on local soft-delete
- [ ] Network failure does not crash the app or block the UI
- [ ] Bad API key disables auto-sync and shows clear error
- [ ] Sync status icons appear in customer list
- [ ] "Sync Now" button triggers manual sync
- [ ] Frontend receives Tauri events for sync progress
- [ ] No sensitive data (API keys, tokens) appears in logs or frontend

## Out of Scope

- Recurring schedule sync (future feature, depends on card-on-file)
- Conflict resolution UI (auto-resolve with last-write-wins for now)
- Webhook-based real-time sync from gateway to local
- Multi-gateway support (Sola only for now)

## Dependencies

- Sola API key must be configured in Settings (existing feature)
- `application-logging-system-with-admin-viewer` — recommended to implement first for audit trail

## Open Questions

- Should we allow the merchant to choose which direction wins in a conflict? (For now: last-write-wins based on timestamps)
- Should we use `CustomerNumber` (our UUID) or `CustomerCustom01` for cross-referencing? (Recommendation: `CustomerNumber` — it's designed for external system IDs)

## API Reference

- **Base URL:** `https://api.cardknox.com/v2`
- **Auth:** `Authorization` header with API key
- **Endpoints used:** `/CreateCustomer`, `/UpdateCustomer`, `/GetCustomer`, `/DeleteCustomer`, `/ListCustomers`
- **Pagination:** `PageSize` (max 100), `NextToken` for cursoring
- **Versioning:** `X-Recurring-Api-Version: 2.1` header

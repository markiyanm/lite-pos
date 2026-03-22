# Implementation Plan: Customer Gateway Sync

## Phase 1: Database & Types
1. Create `src-tauri/migrations/010_customer_gateway_sync.sql`
   - Add 4 columns to customers table
   - Create sync_log table
   - Seed gateway sync settings
2. Register migration in `lib.rs`
3. Update `Customer` TypeScript interface with new fields

## Phase 2: Rust API Client
1. Create `src-tauri/src/recurring_api.rs`
   - Structs for request/response matching Sola Recurring API v2
   - HTTP client with proper headers (Authorization, X-Recurring-Api-Version, SoftwareName)
   - Methods: list_customers, create_customer, update_customer, delete_customer, get_customer
   - Pagination support via NextToken
   - Error handling with typed errors

## Phase 3: Rust Sync Engine
1. Create `src-tauri/src/customer_sync.rs`
   - Tauri command: `sync_customers_with_gateway`
   - Reads API key from DB via tauri-plugin-sql
   - Bidirectional sync with field mapping
   - Tauri event emission for progress
   - Sync lock (AtomicBool) to prevent concurrent runs
   - Delete handling per FR-9/10/11
2. Register command in `lib.rs`

## Phase 4: Frontend Wiring
1. Create `src/lib/commands/customer-sync.ts`
   - `triggerCustomerSync()` - invoke Tauri command
   - `listenSyncEvents()` - listen to gateway-sync-* events
   - Types for sync event payloads
2. Update `src/routes/customers/+page.svelte`
   - Import CloudCheck, CloudUpload, CloudAlert from lucide-svelte
   - Add sync status icon column in table
   - Listen for sync events to auto-refresh
3. Update `src/routes/customers/[id]/+page.svelte`
   - Add sync status badge in sidebar
   - Show last synced timestamp
4. Update `src/routes/settings/+page.svelte`
   - Add "Customer Sync" section in Payments tab (visible only when API key set)
   - Toggle for gateway_customer_sync_enabled
   - Sync interval input
   - "Sync Now" button with loading state
5. Wire background sync listener in `+layout.svelte`
   - On mount, check settings and start listening
   - Toast on sync errors

## Phase 5: Review & Testing
1. Verify graceful failure on network errors
2. Verify safe delete logic
3. Verify API key never exposed
4. Run `npm run check` and `cargo check`

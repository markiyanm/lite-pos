# Implementation Plan: Card Not Present Payments via iFields

## Step 1: Migration - Add ifields_key setting
- File: `src-tauri/migrations/010_ifields_key.sql`
- Register in `src-tauri/src/lib.rs`

## Step 2: Rust command - process_sola_cnp_transaction
- File: `src-tauri/src/sola.rs`
- Add `SolaCnpTransactionRequest` struct
- Add `process_sola_cnp_transaction` command (POST to x1.cardknox.com/gateway)
- Register in lib.rs invoke_handler

## Step 3: Frontend command wrapper
- File: `src/lib/commands/sola.ts`
- Add `processSolaCnpTransaction()` function

## Step 4: iFields script loader utility
- File: `src/lib/utils/ifields-loader.ts`
- Singleton pattern, dynamic script injection, error handling

## Step 5: Settings page - iFields key input
- File: `src/routes/settings/+page.svelte`
- Add iFields key field (encrypted, visibility toggle) in Sola Gateway card
- Save iFields key alongside other gateway settings

## Step 6: PaymentModal - CNP form
- File: `src/lib/components/pos/PaymentModal.svelte`
- Replace placeholder with iFields form
- Card number + CVV iframes, expiry/name/ZIP inputs
- Card brand detection, validation, processing flow
- Token retrieval -> gateway call -> result handling

## Step 7: Type updates
- File: `src/lib/types/index.ts`
- Add `SolaCnpTransactionResult` if needed (reuse existing types where possible)

## Testing
- `npm run check` for TypeScript
- `cargo check` in src-tauri/ for Rust
- Manual verification of iFields loading in Tauri webview

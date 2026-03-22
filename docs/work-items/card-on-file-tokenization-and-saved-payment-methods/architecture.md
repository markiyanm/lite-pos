# Architecture: Card on File Tokenization and Saved Payment Methods

## Overview

This feature adds the ability to securely save customer payment card tokens and use them for future transactions. It spans all layers: database (new table), Rust backend (new gateway commands), TypeScript command layer (CRUD + encryption), and Svelte UI (customer detail + payment modal).

## Data Flow

```
[iFields iframe] -> single-use token (SUT)
       |
       v
[Rust: sola_save_card] -> POST cc:save to gateway -> reusable xToken
       |
       v
[TS: createPaymentToken] -> encrypt_value(xToken) -> INSERT into customer_payment_tokens
       |
       v
[Pay with token] -> SELECT from customer_payment_tokens -> decrypt_value(token)
       |
       v
[Rust: process_sola_token_transaction] -> POST cc:sale with xToken -> standard response
```

## Database Schema

### New Table: `customer_payment_tokens` (Migration 010)

```sql
CREATE TABLE customer_payment_tokens (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uuid TEXT NOT NULL UNIQUE DEFAULT (lower(hex(randomblob(16)))),
  customer_id INTEGER NOT NULL REFERENCES customers(id),
  gateway_payment_method_id TEXT,
  token TEXT NOT NULL,              -- encrypted xToken
  card_last_four TEXT NOT NULL,
  card_brand TEXT,
  exp_month INTEGER,
  exp_year INTEGER,
  is_default INTEGER NOT NULL DEFAULT 0,
  gateway_revision INTEGER,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now')),
  deleted_at TEXT
);
```

### New Setting

- `enable_card_on_file` (boolean, default: `false`, group: `payments`)

## Rust Commands (sola.rs)

### `sola_save_card`

- Sends `cc:save` to `https://x1.cardknox.com/gateway`
- Parameters: apiKey, cardToken, cvvToken, exp, name (optional)
- Returns: SolaSaveCardResult with xToken, xMaskedCardNumber, xCardType

### `process_sola_token_transaction`

- Sends `cc:sale` with `xToken` to `https://x1.cardknox.com/gateway`
- Parameters: apiKey, token, amountCents, invoiceNumber, cvv (optional)
- Returns: SolaTransactionResult (same as existing)

## TypeScript Commands (payment-tokens.ts)

- `getCustomerTokens(customerId)` - SELECT non-deleted tokens
- `createPaymentToken(data)` - INSERT with encrypted token
- `deletePaymentToken(id)` - soft delete + promote default
- `setDefaultToken(customerId, tokenId)` - toggle default
- `getDefaultToken(customerId)` - get default token

## Security Model

1. Raw card numbers never exist in the app (iFields handles PCI scope)
2. Single-use tokens (SUTs) from iFields are ephemeral, used only for cc:save
3. Reusable xTokens are encrypted with AES-256-GCM before DB storage
4. Decryption happens only in the Rust backend for gateway calls
5. Frontend receives only: last_four, brand, exp_month, exp_year, is_default
6. Tokens are never logged (no println! for token values)

## UI Components

### Customer Detail Page (`/customers/[id]`)

- New "Payment Methods" section below order history
- Card list with brand icon, masked number, expiry, default badge
- "Add Card" button opens dialog with iFields iframes
- Delete button with confirmation dialog

### PaymentModal

- "Save card for future use" checkbox (when customer attached + setting enabled)
- "Saved Cards" section (when customer has tokens)
- Token payment flow using process_sola_token_transaction

### Settings Page

- New toggle: "Enable Card on File" in Payments tab
- Gated on both sola_gateway_api_key and ifields_key being configured

## Dependencies

- iFields library (for add-card dialog) - not yet implemented, stubbed
- Customer gateway sync (for gateway_customer_id) - not yet implemented, stubbed
- Recurring API integration deferred to avoid scope creep

## Scope Decisions

1. Gateway recurring API (CreatePaymentMethod, ListPaymentMethods, DeletePaymentMethod) is deferred. Tokens are stored locally only. Gateway sync will be added when the customer-gateway-sync feature lands.
2. iFields integration in the add-card dialog is stubbed since the CNP feature hasn't landed yet. The dialog structure and token handling are ready.
3. `gateway_customer_id` column on customers table is NOT added here - that belongs to the customer-gateway-sync feature.

# Requirements: Card on File Tokenization and Saved Payment Methods

## Problem Statement

Repeat customers currently need to present their card for every transaction. Merchants want to securely store card tokens for faster repeat sales, especially for regulars and phone orders. Sola's gateway supports tokenization via `cc:save` (explicit save without a transaction) and returns reusable `xToken` values during regular transactions. Combined with iFields for initial card capture and the Sola recurring API for payment method management, Vira can offer a complete card-on-file experience.

## User Stories

- As a cashier, I want to save a customer's card during checkout so future transactions are faster.
- As a cashier, I want to see a customer's saved cards and charge them without re-entering card details.
- As a cashier, I want to add a card to a customer's file without processing a sale (e.g., for future phone orders).
- As a customer, I want my card data to be stored securely (tokenized, not the actual number).
- As a merchant, I want to control whether card-on-file is enabled via Settings.
- As a cashier, I want to delete a saved card that's expired or no longer valid.

## Functional Requirements

### Settings

1. **FR-1:** Add boolean setting `enable_card_on_file` in the Payments settings tab. Default: `false` (off — merchant must explicitly opt in).
2. **FR-2:** Card-on-file requires both `sola_gateway_api_key` AND `ifields_key` to be configured. If either is missing, show a warning message explaining what's needed.

### Database Schema (Migration 011)

3. **FR-3:** Create a `customer_payment_tokens` table:
   ```sql
   CREATE TABLE customer_payment_tokens (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     uuid TEXT NOT NULL UNIQUE DEFAULT (lower(hex(randomblob(16)))),
     customer_id INTEGER NOT NULL REFERENCES customers(id),
     gateway_payment_method_id TEXT,          -- Sola PaymentMethodId
     token TEXT NOT NULL,                      -- xToken from gateway (encrypted at rest)
     card_last_four TEXT NOT NULL,             -- e.g., "4242"
     card_brand TEXT,                          -- Visa, Mastercard, Amex, Discover
     exp_month INTEGER,                        -- 1-12
     exp_year INTEGER,                         -- 4-digit year
     is_default INTEGER NOT NULL DEFAULT 0,   -- boolean
     gateway_revision INTEGER,                 -- Sola revision for updates
     created_at TEXT NOT NULL DEFAULT (datetime('now')),
     updated_at TEXT NOT NULL DEFAULT (datetime('now')),
     deleted_at TEXT                            -- soft delete
   );
   CREATE INDEX idx_cpt_customer_id ON customer_payment_tokens(customer_id);
   ```
4. **FR-4:** Token values must be encrypted using the existing `encrypt_value` Tauri command before storage and decrypted with `decrypt_value` when needed for transactions.

### Save Card During Checkout

5. **FR-5:** In the PaymentModal, when a customer is attached to the order and `enable_card_on_file` is true, show a checkbox: "Save card for future use".
6. **FR-6:** The checkbox is only visible for credit card payments (both card-present and card-not-present).
7. **FR-7:** When the checkbox is checked and the transaction succeeds:
   a. Extract `xToken`, `xMaskedCardNumber`, `xCardType`, `xExp` from the gateway response
   b. Parse last four digits from `xMaskedCardNumber`
   c. Parse exp_month and exp_year from `xExp` (MMYY format)
   d. Create a record in `customer_payment_tokens` with the encrypted token
   e. If the customer has a `gateway_customer_id`, also call Sola recurring API `CreatePaymentMethod` with the token to register it on the gateway side. Store the returned `PaymentMethodId`.
8. **FR-8:** If saving the token fails (DB error or gateway error), log the error but do NOT fail the transaction. The payment was already successful.

### Save Card Without Transaction (Customer Detail View)

9. **FR-9:** On the customer detail page (`/customers/[id]`), add a "Payment Methods" section below the order history.
10. **FR-10:** Add an "Add Card" button that opens a dialog with:
    - iFields card number iframe
    - iFields CVV iframe
    - Expiration date input
    - Cardholder name input
    - "Set as default" checkbox
11. **FR-11:** On submit:
    a. Call `getTokens()` via iFields to get the single-use token
    b. Call Sola `cc:save` via a new Tauri command `sola_save_card(apiKey, cardToken, cvvToken, exp, name?)`:
       - `xCommand: cc:save`
       - `xCardNum: <cardToken>`
       - `xCVV: <cvvToken>`
       - `xExp: <MMYY>`
       - `xName: <name>`
       - `xKey: <apiKey>`
       - Response returns `xToken` (reusable), `xMaskedCardNumber`, `xCardType`
    c. If customer has `gateway_customer_id`, call recurring API `CreatePaymentMethod` with the token
    d. Store in `customer_payment_tokens` (encrypted)
12. **FR-12:** If the customer doesn't have a `gateway_customer_id` yet, create the customer on the gateway first (using the customer sync logic from the customer-gateway-sync feature, or a direct `CreateCustomer` call), then create the payment method.

### List Saved Cards

13. **FR-13:** In the customer detail "Payment Methods" section, show a card list with:
    - Card brand icon (Visa, Mastercard, Amex, Discover)
    - Masked number: `•••• •••• •••• 4242`
    - Expiration: `MM/YY`
    - "Default" badge on the default card
    - "Delete" button per card
14. **FR-14:** On page load, if the customer has a `gateway_customer_id`, also fetch payment methods from the Sola recurring API (`ListPaymentMethods` filtered by `CustomerId`) and reconcile:
    - Gateway methods not in local DB: create local records
    - Local methods not on gateway: push to gateway (if `gateway_customer_id` exists)
    - Mismatched data: gateway is source of truth for card details; local is source of truth for `is_default`

### Pay with Saved Card

15. **FR-15:** In the PaymentModal, when a customer is attached and has saved cards, show a "Saved Cards" section above the card-present/key-in options.
16. **FR-16:** Each saved card shows: brand icon, last four, expiration. Tapping selects it.
17. **FR-17:** When paying with a saved card:
    a. Decrypt the token from `customer_payment_tokens`
    b. Call a new Tauri command `process_sola_token_transaction(apiKey, token, amountCents, invoiceNumber, cvv?)`:
       - `xCommand: cc:sale`
       - `xToken: <decryptedToken>`
       - `xAmount: <dollars>`
       - `xInvoice: <invoiceNumber>`
       - `xKey: <apiKey>`
       - CVV is optional for token transactions but recommended if available
    c. Process response same as existing payment flow
18. **FR-18:** Store payment with `card_entry_mode = 'token'` to distinguish from other entry methods.

### Delete Saved Card

19. **FR-19:** "Delete" button on a saved card shows confirmation dialog: "Remove this card ending in 4242?"
20. **FR-20:** On confirm:
    a. If the card has a `gateway_payment_method_id`, call Sola recurring API `DeletePaymentMethod`
    b. Soft-delete the local record (`deleted_at`)
    c. If gateway delete fails (e.g., active schedule), warn the user but still soft-delete locally
21. **FR-21:** Deleting the default card automatically promotes the next most-recently-created card to default.

### Commands Module

22. **FR-22:** Create `src/lib/commands/payment-tokens.ts`:
    - `getCustomerTokens(customerId)` — list non-deleted tokens for a customer
    - `createPaymentToken(data)` — insert new token record
    - `deletePaymentToken(id)` — soft-delete
    - `setDefaultToken(customerId, tokenId)` — set one as default, unset others
    - `getDefaultToken(customerId)` — get the default token for a customer

## Non-Functional Requirements

- **Security:** Tokens stored encrypted at rest. Decrypted only in Rust backend for gateway calls. Never sent to frontend in plaintext. Never logged.
- **PCI Compliance:** Raw card numbers never exist in the app. Only iFields tokens (single-use) and gateway tokens (reusable, encrypted).
- **Performance:** Saved card list should load < 500ms including gateway reconciliation.
- **Data Integrity:** Failed token saves must not fail successful transactions.

## Acceptance Criteria

- [ ] Setting to enable/disable card-on-file exists, defaults to off
- [ ] Setting requires both API key and iFields key to be configured
- [ ] "Save card" checkbox appears in PaymentModal when customer is attached and setting is enabled
- [ ] Successful transaction with checkbox stores encrypted token in DB
- [ ] Customer detail page shows "Payment Methods" section with saved cards
- [ ] "Add Card" dialog uses iFields iframes and saves via cc:save
- [ ] Cards saved locally are synced to gateway via recurring API CreatePaymentMethod
- [ ] Gateway payment methods not in local DB are pulled on page load
- [ ] PaymentModal shows "Saved Cards" section when customer has tokens
- [ ] Paying with saved card uses token (no card re-entry needed)
- [ ] Payment stored with `card_entry_mode = 'token'`
- [ ] Delete removes from both local DB and gateway
- [ ] Deleting default card promotes next card to default
- [ ] Token values are encrypted in DB and decrypted only in Rust
- [ ] No raw card numbers or tokens appear in logs
- [ ] Receipt prints correctly for token transactions

## Out of Scope

- Card verification (AVS/CVV check without a charge) — just save and go
- Automatic token refresh for expired cards
- Multi-gateway token management (Sola only)
- Card-on-file for ACH/bank accounts
- Recurring schedule creation (future feature)

## Dependencies

- `card-not-present-payments-via-ifields-integration` — required for the iFields iframe used in "Add Card" dialog
- `customer-gateway-sync-with-sola-recurring-api` — recommended for `gateway_customer_id` to be populated. Without it, payment methods won't be linked to gateway customers.
- `application-logging-system-with-admin-viewer` — recommended for audit trail of card saves/deletes

## Open Questions

- Should CVV be required when paying with a saved card? (Recommendation: optional — some merchants may not want the friction, but it reduces fraud risk. Make it a setting in the future.)
- The Sola docs reference `DeletePaymentMethod` with a constraint about active schedules. Since we don't support schedules yet, deletion should always succeed. Confirm with Sola that tokenized cards without schedules can always be deleted.

## API Reference

### Transaction API (cc:save)
- **URL:** `https://x1.cardknox.com/gateway`
- **Method:** POST
- **Fields:** `xKey`, `xVersion: 5.0.0`, `xCommand: cc:save`, `xCardNum` (SUT from iFields), `xCVV` (SUT), `xExp` (MMYY), `xSoftwareName: Vira`, `xSoftwareVersion`
- **Response:** `xToken` (reusable), `xMaskedCardNumber`, `xCardType`, `xResult`, `xRefNum`

### Transaction API (cc:sale with token)
- **URL:** `https://x1.cardknox.com/gateway`
- **Method:** POST
- **Fields:** `xKey`, `xVersion: 5.0.0`, `xCommand: cc:sale`, `xToken`, `xAmount`, `xInvoice`, `xSoftwareName: Vira`, `xSoftwareVersion`
- **Response:** Standard transaction response

### Recurring API (Payment Methods)
- **Base URL:** `https://api.cardknox.com/v2`
- **Endpoints:** `/CreatePaymentMethod`, `/ListPaymentMethods`, `/DeletePaymentMethod`, `/GetPaymentMethod`
- **Auth:** `Authorization` header with API key
- **Required for create:** `CustomerId`, `Token`, `TokenType: cc`, `Exp` (MMYY)

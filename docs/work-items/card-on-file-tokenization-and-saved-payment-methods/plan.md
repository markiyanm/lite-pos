# Implementation Plan: Card on File Tokenization

## Phase 1: Database & Settings

1. **Migration 010**: Create `customer_payment_tokens` table + index + updated_at trigger
2. **Migration 010**: Insert `enable_card_on_file` setting (boolean, default false, group payments)
3. Register migration in `lib.rs`

## Phase 2: Rust Backend

4. **sola.rs**: Add `SolaSaveCardRequest` and `SolaSaveCardResult` structs
5. **sola.rs**: Add `sola_save_card` command (POST cc:save to gateway)
6. **sola.rs**: Add `SolaTokenTransactionRequest` struct
7. **sola.rs**: Add `process_sola_token_transaction` command (POST cc:sale with xToken)
8. **lib.rs**: Register both new commands in invoke_handler

## Phase 3: TypeScript Types & Commands

9. **types/index.ts**: Add `CustomerPaymentToken` interface
10. **types/index.ts**: Add `SolaSaveCardResult` type
11. **commands/sola.ts**: Add `solaSaveCard()` and `processSolaTokenTransaction()` wrappers
12. **commands/payment-tokens.ts**: Create new module with CRUD functions:
    - `getCustomerTokens(customerId)`
    - `createPaymentToken(data)` - encrypts token before storage
    - `deletePaymentToken(id)` - soft delete + default promotion
    - `setDefaultToken(customerId, tokenId)`
    - `getDefaultToken(customerId)`

## Phase 4: Settings UI

13. **settings/+page.svelte**: Add `enable_card_on_file` toggle in Payments tab
14. Include prerequisite warning (requires API key + iFields key)
15. Wire up save logic

## Phase 5: Customer Detail - Payment Methods Section

16. **customers/[id]/+page.svelte**: Add "Payment Methods" card section
17. Load saved cards on mount via `getCustomerTokens()`
18. Display card list with brand, masked number, expiry, default badge
19. Add "Set as Default" action
20. Add "Delete" button with confirmation dialog
21. Add "Add Card" button (stubbed - dialog placeholder until iFields lands)

## Phase 6: PaymentModal - Save Card Checkbox

22. **PaymentModal.svelte**: Accept `customerId` prop
23. Add "Save card for future use" checkbox (visible when customer attached + setting enabled + card payment)
24. After successful card-present transaction, if checkbox checked:
    - Extract xToken from response (add xToken to SolaTransactionResponse type)
    - Create payment token record via `createPaymentToken()`

## Phase 7: PaymentModal - Pay with Saved Card

25. **PaymentModal.svelte**: Load customer's saved cards when modal opens (if customer attached)
26. Add "Saved Cards" section with card selection UI
27. Implement token payment flow:
    - Select saved card
    - Call `process_sola_token_transaction` via Rust
    - Handle response same as existing flow
    - Store payment with `card_entry_mode = 'token'`

## Commit Strategy

Each phase gets its own commit with `[CardOnFile]` prefix:
- `[CardOnFile] Add customer_payment_tokens migration and setting`
- `[CardOnFile] Add Rust commands for cc:save and token transactions`
- `[CardOnFile] Add TypeScript types and payment-tokens commands`
- `[CardOnFile] Add card-on-file toggle to Payments settings`
- `[CardOnFile] Add Payment Methods section to customer detail page`
- `[CardOnFile] Add save-card checkbox to PaymentModal`
- `[CardOnFile] Add pay-with-saved-card flow to PaymentModal`

## Testing Checklist

- [ ] `cargo check` passes with new Rust commands
- [ ] `npm run check` passes with new TS types
- [ ] Token encryption/decryption via encrypt_value/decrypt_value
- [ ] Migration creates table correctly
- [ ] Setting defaults to false
- [ ] Payment token CRUD operations
- [ ] Soft delete promotes default card

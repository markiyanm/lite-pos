# Requirements: Card Not Present Payments via iFields Integration

## Problem Statement

Vira currently supports card-present payments via Sola terminal devices but has no way to process card-not-present (CNP) transactions. The PaymentModal already has a placeholder for CNP ("coming soon"). Merchants need to process phone orders, manual entry for damaged magstripes, and online-style keyed transactions. iFields provides a PCI-compliant hosted iframe solution so the merchant's system never touches raw card data.

## User Stories

- As a cashier, I want to manually enter a customer's card number for phone orders or when the card can't be swiped/tapped.
- As a merchant, I want manual card entry to be PCI compliant so I don't have liability for card data.
- As a cashier, I want the key-in flow to feel like part of the existing payment experience.
- As a merchant, I want to control whether manual card entry is allowed in my store.

## Functional Requirements

### Settings

1. **FR-1:** The existing `sola_gateway_card_not_present` boolean setting already exists (migration 003). Ensure it is wired to the toggle in the Payments settings tab. Default: `true` (allowed).
2. **FR-2:** Add a new encrypted setting `ifields_key` for the merchant's iFields public key. This is separate from the Sola API key. Show a help text explaining where to find it in the Sola Portal.
3. **FR-3:** CNP payments should only be available when both `sola_gateway_card_not_present` is true AND `ifields_key` is configured.

### iFields Library Loading

4. **FR-4:** Load iFields JS from CDN: `https://cdn.cardknox.com/ifields/3.4.2602.2001/ifields.min.js`. Do NOT use npm packages.
5. **FR-5:** Since Vira is a Tauri app (webview), load the script dynamically when the PaymentModal opens in CNP mode:
   ```typescript
   const script = document.createElement('script');
   script.src = 'https://cdn.cardknox.com/ifields/3.4.2602.2001/ifields.min.js';
   document.head.appendChild(script);
   ```
6. **FR-6:** After script loads, initialize with: `setAccount(ifieldKey, "Vira", version)`.
7. **FR-7:** Cache the script load — only inject once per app session.

### Payment Modal Integration

8. **FR-8:** When the cashier selects "Credit Card" payment method and then chooses "Key In" (card not present), show the iFields form instead of the device selection.
9. **FR-9:** The iFields form contains:
   - Card number iframe (`data-ifields-id="card-number"`)
   - CVV iframe (`data-ifields-id="cvv"`)
   - Expiration date input (standard HTML input, NOT an iframe — iFields doesn't iframe this)
   - Cardholder name input (standard HTML input)
   - Billing ZIP code input (for AVS verification)
   - Hidden token inputs (`data-ifields-id="card-number-token"`, `data-ifields-id="cvv-token"`)
10. **FR-10:** Style the iframes to match Vira's UI theme using `setIfieldStyle()`:
    - Match the existing input border, font-size, padding, border-radius
    - Apply dark mode styles when dark theme is active
11. **FR-11:** Enable auto-formatting on the card number field: `enableAutoFormatting('-')`.
12. **FR-12:** Use `addIfieldKeyPressCallback` to:
    - Show real-time card brand icon (Visa, Mastercard, Amex, Discover) based on `data.issuer`
    - Enable/disable the "Pay" button based on `data.cardNumberIsValid && data.cvvIsValid`
    - Show validation feedback (green border when valid, red when invalid)

### Token Flow

13. **FR-13:** When the cashier clicks "Process Payment":
    a. Call `getTokens(onSuccess, onError, 30000)` to retrieve single-use tokens
    b. On success: hidden inputs are populated with tokens
    c. Send tokens to Sola gateway via a new Tauri command `process_sola_cnp_transaction`
14. **FR-14:** New Rust command `process_sola_cnp_transaction(apiKey, cardToken, cvvToken, exp, amount_cents, invoiceNumber, name?, zip?)`:
    - POST to `https://x1.cardknox.com/gateway` (standard transaction endpoint, not device endpoint)
    - `xCommand: cc:sale`
    - `xCardNum: <cardToken>` (the iFields SUT)
    - `xCVV: <cvvToken>` (the iFields SUT)
    - `xExp: <MMYY>`
    - `xAmount: <dollars>`
    - `xInvoice: <invoiceNumber>`
    - `xName: <name>` (optional)
    - `xZip: <zip>` (optional, for AVS)
    - `xKey: <apiKey>`
    - `xVersion: 5.0.0`
    - `xSoftwareName: Vira`
    - `xSoftwareVersion: <version>`
15. **FR-15:** Response handling matches existing card-present flow:
    - `xResult: A` → approved, store auth code, masked card, card type
    - `xResult: D` → declined, show decline message
    - `xResult: E` → error, show error message
    - Store `xToken` from response for potential future card-on-file use
16. **FR-16:** Store payment record with `card_entry_mode = 'keyed'` to distinguish from card-present (`swiped`, `tapped`, `inserted`).

### Error Handling

17. **FR-17:** If iFields script fails to load (no internet, CDN down), show error message: "Card entry is unavailable. Please check your internet connection or use another payment method."
18. **FR-18:** If `getTokens()` times out (30s), show: "Card processing timed out. Please try again."
19. **FR-19:** If the gateway declines, show the decline reason from `xError` and allow the cashier to retry or choose a different payment method.

### UI/UX Details

20. **FR-20:** The card type selector in the existing PaymentModal ("Card Present" / "Key In") should be a segmented button group, not a dropdown.
21. **FR-21:** Show a lock icon and "Secure Entry" label near the iFields form to reassure the cashier that card data is handled securely.
22. **FR-22:** While `getTokens()` and the gateway call are in progress, show a spinner with "Processing payment..." and disable the form.

## Non-Functional Requirements

- **Security/PCI:** Raw card numbers must NEVER exist in the app's memory, logs, or database. Only iFields tokens and masked card numbers.
- **Performance:** iFields script should be pre-loaded when the app detects CNP is enabled, not on first modal open.
- **Compatibility:** Must work in Tauri's WebView2 (Windows) and WebKit (macOS/Linux) webviews.
- **Offline:** CNP is inherently online-only. If no internet, the option should be grayed out with a tooltip.

## Acceptance Criteria

- [ ] Setting to enable/disable CNP exists and respects `sola_gateway_card_not_present`
- [ ] iFields key can be configured in Settings (encrypted storage)
- [ ] CNP option appears in payment modal when enabled and iFields key is set
- [ ] Card number and CVV render in iframes (verify via DevTools — no card data in main DOM)
- [ ] Card brand icon updates in real-time as card number is typed
- [ ] Pay button is disabled until card number and CVV are valid
- [ ] Successful payment stores auth code, masked card, card type, entry mode = 'keyed'
- [ ] Declined payment shows decline reason
- [ ] iFields script load failure shows clear error message
- [ ] No raw card numbers appear in logs, database, or Rust backend at any point
- [ ] Dark mode styling is applied to iFields iframes
- [ ] Receipt prints correctly for CNP transactions (shows "Key In" as entry method)

## Out of Scope

- 3DS / 3D Secure verification (explicitly excluded per user request)
- ACH/bank account payments via iFields
- Address verification beyond ZIP code
- Recurring/subscription payments (handled by card-on-file feature)
- Refunds for CNP transactions (existing refund flow should work with `xRefNum`)

## Dependencies

- `sola_gateway_api_key` must be configured (existing)
- `ifields_key` must be configured (new setting, this feature)
- Internet connectivity required for both iFields iframe loading and gateway processing

## Open Questions

- None. The iFields CDN version is pinned to 3.4.2602.2001. Upgrade path: change the version constant in one place.

## Technical References

- iFields JS: `https://cdn.cardknox.com/ifields/3.4.2602.2001/ifields.min.js`
- iFrame source: `https://cdn.cardknox.com/ifields/3.4.2602.2001/ifield.htm`
- Transaction gateway: `https://x1.cardknox.com/gateway` (POST, form-encoded or JSON)
- Key functions: `setAccount()`, `setIfieldStyle()`, `enableAutoFormatting()`, `addIfieldKeyPressCallback()`, `getTokens()`, `focusIfield()`, `clearIfield()`

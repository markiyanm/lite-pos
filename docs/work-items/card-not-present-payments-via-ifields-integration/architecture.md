# Architecture: Card Not Present Payments via iFields Integration

## Overview

Add CNP (card-not-present) payment capability using Cardknox iFields hosted iframes for PCI-compliant card data entry. The iFields iframes handle raw card data in isolated contexts, the app only ever receives single-use tokens (SUTs).

## Data Flow

```
User types card in iFrame -> iFields JS SDK -> getTokens() -> SUT tokens
-> Tauri command (process_sola_cnp_transaction) -> POST https://x1.cardknox.com/gateway
-> Response (approved/declined/error) -> Store payment record with entry_mode='keyed'
```

## Components

### 1. Migration (010_ifields_key.sql)
- Adds `ifields_key` setting to the settings table (encrypted, gateway group)

### 2. Rust Backend (sola.rs)
- New struct `SolaCnpTransactionRequest` with card token, CVV token, exp, amount, etc.
- New command `process_sola_cnp_transaction` that POSTs form-encoded data to `https://x1.cardknox.com/gateway`
- Reuses existing `SolaTransactionResponse` and `SolaTransactionResult` types
- New `SolaCnpRequestInfo` variant for request debugging

### 3. Frontend Command Wrapper (sola.ts)
- New `processSolaCnpTransaction()` function wrapping the Tauri invoke

### 4. iFields Loader Utility (src/lib/utils/ifields-loader.ts)
- Singleton script loader: injects CDN script once, caches load promise
- Exports `loadIfields()`, `isIfieldsLoaded()` functions
- Handles load errors gracefully

### 5. PaymentModal CNP Form
- Replace "coming soon" placeholder with iFields form
- Card number iframe + CVV iframe + expiry input + name input + ZIP input
- Card brand detection via `addIfieldKeyPressCallback`
- Validation states (green/red borders)
- Token retrieval via `getTokens()` -> gateway submission

### 6. Settings Page
- Add iFields Key input field in the Payments > Sola Gateway section
- Encrypted storage (same pattern as API key)

## Security Boundaries

- Raw card numbers: ONLY inside iFields iframes (cross-origin, inaccessible to app)
- App receives: single-use tokens only
- Tokens sent: to Rust backend via Tauri IPC (in-process, not network)
- Rust backend: forwards tokens to gateway over HTTPS, never logs card data
- Database: stores only masked card number, card type, auth code, entry mode

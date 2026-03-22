# Design: Card on File Tokenization UI

## 1. Customer Detail Page - Payment Methods Section

Located below the Order History section on `/customers/[id]`.

### Card List
```
+----------------------------------------------------------+
| Payment Methods                              [Add Card]  |
+----------------------------------------------------------+
| VISA  **** **** **** 4242   Exp 03/28   [Default]  [x]  |
| MC    **** **** **** 8888   Exp 12/27              [x]  |
+----------------------------------------------------------+
| No saved payment methods.              (empty state)     |
+----------------------------------------------------------+
```

- Uses shadcn `Card` with `CardHeader` (title + Add Card button) and `CardContent`
- Each saved card is a row with: brand icon (Lucide CreditCard), masked number, expiry, default Badge, delete Button
- Brand icons: text labels (Visa, MC, Amex, Disc) with appropriate styling
- Default card gets a `Badge` with "Default" text
- Delete button: ghost variant, Trash2 icon, triggers confirmation AlertDialog
- Empty state: centered muted text

### Delete Confirmation Dialog
```
+-------------------------------------------+
| Remove Payment Method?                    |
|                                           |
| Remove card ending in 4242? This cannot   |
| be undone.                                |
|                                           |
|              [Cancel]  [Remove]           |
+-------------------------------------------+
```

- Uses shadcn `AlertDialog`
- Destructive action button

### Add Card Dialog (Stubbed)
```
+-------------------------------------------+
| Add Payment Method                        |
|                                           |
| Card Number                               |
| [iFields iframe placeholder]              |
|                                           |
| CVV                                       |
| [iFields iframe placeholder]              |
|                                           |
| Expiration    MM / YY                     |
| Cardholder Name                           |
| [________________________]                |
|                                           |
| [ ] Set as default payment method         |
|                                           |
|              [Cancel]  [Save Card]        |
+-------------------------------------------+
```

- Uses shadcn `Dialog`
- iFields iframes are placeholder divs with dashed border until CNP feature lands
- Expiration: two small inputs (MM and YY) side by side
- Cardholder name: standard Input
- Default checkbox: shadcn Checkbox
- Save button disabled until iFields integration is ready

## 2. PaymentModal - Save Card Checkbox

Added to the card payment section (both card_present and card_not_present flows).

Visibility conditions:
- `enable_card_on_file` setting is true
- A customer is attached to the current order
- Payment method is credit_card

```
+-------------------------------------------+
| Card Present / Key In toggle              |
|                                           |
| [Terminal Device selector]                |
| [Amount input]                            |
|                                           |
| [x] Save card for future use             |
|                                           |
|        [Cancel]  [Send to Device]         |
+-------------------------------------------+
```

- Simple shadcn Checkbox + Label below the amount input
- Checked state stored in component, passed to onComplete callback

## 3. PaymentModal - Saved Cards Section

Shown above the card-present/key-in options when customer has saved cards.

```
+-------------------------------------------+
| Saved Cards                               |
|                                           |
| ( ) VISA **** 4242  03/28  [Default]     |
| ( ) MC   **** 8888  12/27                |
|                                           |
| -- or pay with new card --                |
|                                           |
| Card Present  |  Key In                   |
+-------------------------------------------+
```

- Radio-style selection using shadcn Button with outline/default variants
- Selected card highlighted with default variant
- Separator with "or pay with new card" text
- When a saved card is selected, the action button becomes "Pay with Saved Card"
- Amount input still shown (defaults to exact remaining)

## 4. Settings - Card on File Toggle

Added to the Payments tab, inside the Sola Gateway card, after the Card Not Present toggle.

```
+-------------------------------------------+
| Sola Gateway                              |
|                                           |
| API Key: [*************]                  |
| ---------------------------------------- |
| Card Not Present              [toggle]    |
| Card Present                  [toggle]    |
| ---------------------------------------- |
| Card on File                  [toggle]    |
| Save and reuse customer card tokens       |
| for faster repeat transactions.           |
|                                           |
| (!) Requires API Key to be configured.    |
+-------------------------------------------+
```

- Uses existing `SettingToggle` component
- Warning message shown if API key is not configured
- Toggle disabled if no API key

## Component Hierarchy

```
PaymentModal.svelte
  +-- SavedCardSelector.svelte (new, extracted for clarity)

customers/[id]/+page.svelte
  +-- PaymentMethodsList.svelte (new component)
  +-- AddCardDialog.svelte (new component, stubbed)
  +-- DeleteCardDialog.svelte (uses AlertDialog)
```

## Color & Style Notes

- Card brand colors: not used (keep monochrome for consistency)
- Expired cards: show `text-destructive` on expiry date
- Default badge: `variant="secondary"` Badge
- All follows existing Tailwind v4 patterns with shadcn-svelte components

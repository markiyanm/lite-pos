# Design: Card Not Present iFields Form

## CNP Form Layout (within PaymentModal)

When "Key In" is selected:

```
+------------------------------------------+
| [Lock icon] Secure Card Entry             |
+------------------------------------------+
| Amount: $XX.XX                            |
+------------------------------------------+
| Card Number          [Visa/MC/Amex icon]  |
| +--------------------------------------+  |
| | [iFields iframe - card-number]       |  |
| +--------------------------------------+  |
|                                           |
| Expiration    CVV                         |
| +----------+  +------------------------+ |
| | MM / YY  |  | [iFields iframe - cvv] | |
| +----------+  +------------------------+ |
|                                           |
| Cardholder Name                           |
| +--------------------------------------+  |
| | John Doe                             |  |
| +--------------------------------------+  |
|                                           |
| Billing ZIP                               |
| +--------------------------------------+  |
| | 10001                                |  |
| +--------------------------------------+  |
+------------------------------------------+
| [Cancel]              [Process Payment]   |
+------------------------------------------+
```

## Validation States
- **Invalid**: Red border on iframe/input, red text helper
- **Valid**: Green border (subtle), checkmark
- **Processing**: All inputs disabled, spinner overlay with "Processing payment..."

## Card Brand Icons
- Detected via `addIfieldKeyPressCallback` -> `data.issuer`
- Show: Visa, Mastercard, Amex, Discover, or generic card icon
- Positioned right side of card number label row

## iFields Key Setting (Settings > Payments > Sola Gateway)
- Placed after the API Key field, before the toggles
- Same encrypted input pattern (password field + eye toggle)
- Help text: "Your iFields key from the Sola Portal (Settings > API Keys > iFields)"

## Dark Mode
- iFrame styles applied via `setIfieldStyle()` matching Tailwind theme tokens
- Border, background, text color, font-size matched to shadcn Input component

## Error States
- Script load failure: Alert banner with retry button
- Token timeout: Toast error + form re-enabled
- Gateway decline: Toast with decline reason, form stays open for retry

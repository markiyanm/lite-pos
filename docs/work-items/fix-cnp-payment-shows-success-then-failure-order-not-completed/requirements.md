# Requirements: Fix CNP Payment Shows Success Then Failure, Order Not Completed

GitHub Issue: https://github.com/markiyanm/lite-pos/issues/2

## Problem Statement

When processing a card-not-present (manual key-in) payment, the gateway approves the transaction but the POS shows a success toast followed seconds later by a failure toast. The order remains open/incomplete on the POS even though money was charged on the gateway. This is a P0 bug — the merchant has charged the customer but has no record of the completed order.

## Root Cause Analysis

In `src/lib/components/pos/PaymentModal.svelte`, `handleCnpPayment()`:
1. Line 382: Gateway returns `xResult === "A"` (approved)
2. Line 402: Success toast shown: "Payment approved"
3. Line 408: `onComplete(partialPayments)` called **without `await`**
4. The `onComplete` prop is typed as `(payments: PartialPayment[]) => void` but the parent passes `handlePaymentComplete` which is an `async function`
5. Since the Promise is not awaited, the modal continues execution while the parent's order creation runs asynchronously
6. If the parent's `withTransaction()` fails (stock deduction, order item creation, etc.), line 400 in `+page.svelte` shows "Payment failed" error toast
7. The modal is never closed because `paymentOpen` is only set to `false` on success
8. Result: user sees both toasts, order is not created, but money is charged

The same issue exists for card-present payments at line 566.

## Functional Requirements

1. **FR-1:** Change `onComplete` prop type from `(payments: PartialPayment[]) => void` to `(payments: PartialPayment[]) => void | Promise<void>`.
2. **FR-2:** `await` the `onComplete()` call in `handleCnpPayment()` (line 408) so errors propagate correctly.
3. **FR-3:** `await` the `onComplete()` call in the card-present flow (line 566) for the same reason.
4. **FR-4:** Move the "Payment approved" success toast from the modal to the parent's `handlePaymentComplete` — only show it AFTER the order is successfully created in the database.
5. **FR-5:** In the modal, show an intermediate state ("Processing order...") between gateway approval and order completion.
6. **FR-6:** If the order creation fails after gateway approval, show a clear error: "Payment was approved but order creation failed. Please contact support. Reference: {xRefNum}" — this gives the merchant the gateway reference to reconcile manually.
7. **FR-7:** Log the gateway approval details (ref number, amount, card last 4) before attempting order creation, so there's an audit trail even if the order creation fails.

## Acceptance Criteria

- [ ] CNP payment that succeeds on gateway creates a completed order on the POS
- [ ] Only ONE toast is shown (success) after the entire flow completes
- [ ] The payment modal closes after successful order completion
- [ ] If order creation fails after gateway approval, a specific error with the gateway reference number is shown
- [ ] The gateway approval is logged before order creation is attempted
- [ ] Card-present payments have the same fix (onComplete awaited)
- [ ] No duplicate toasts (success then failure) under any circumstance

## Non-Functional Requirements

- **Data Integrity:** Gateway-approved transactions must always be traceable, even if order creation fails. The log must contain the gateway reference number.
- **UX:** The user must never see contradictory toasts (success + failure).

## Out of Scope

- Automatic reconciliation of orphaned gateway transactions
- Retry logic for failed order creation after approved payment
- Void/refund of the gateway transaction if order creation fails

## Dependencies

- `application-logging-system-with-admin-viewer` — for logging gateway approvals (already implemented)

# Requirements: Fix Draft Save Button Failing with Error Toast

GitHub Issue: https://github.com/markiyanm/lite-pos/issues/1

## Problem Statement

The "Save Draft" button on the POS screen fails with a red error toast ("Failed to save draft"). Drafts should save the current order in a draft state so it can be resumed later. The root cause is that `getNextOrderNumber()` is called outside the `withTransaction()` block in `handleSaveDraft()`, and a database error during order creation causes the entire operation to fail.

## Root Cause Analysis

In `src/routes/pos/+page.svelte`, `handleSaveDraft()`:
1. Line 86: `getNextOrderNumber()` is called and increments the DB counter
2. Line 89: `withTransaction()` starts
3. If `createOrder()` or `addOrderItem()` fails, the transaction rolls back but the order number is already consumed
4. The generic catch block shows "Failed to save draft" without any diagnostic info

The actual failure may be caused by:
- A missing or malformed field in the order data
- The `addOrderItem` call failing on a constraint
- The order number having already been used (UNIQUE constraint violation if retrying)

## Functional Requirements

1. **FR-1:** Move `getNextOrderNumber()` inside the `withTransaction()` block so the number increment is rolled back if order creation fails.
2. **FR-2:** Ensure all required fields for `createOrder()` are populated correctly for draft orders (status = 'draft', no payment required).
3. **FR-3:** Add specific error logging before the generic toast so the actual failure reason is captured in the log system.
4. **FR-4:** After a successful draft save, clear the cart and show a success toast with the order number.
5. **FR-5:** If the draft save fails, show the specific error reason in the toast (not just "Failed to save draft").

## Acceptance Criteria

- [ ] Clicking "Save Draft" with items in the cart creates a draft order successfully
- [ ] A success toast shows with the draft order number
- [ ] The cart is cleared after successful draft save
- [ ] If the save fails, the error toast includes the reason
- [ ] Order numbers are not wasted on failed draft saves (counter only increments inside transaction)
- [ ] The draft appears in the Orders list with status "draft"

## Out of Scope

- Draft resume/edit flow (existing feature)
- Draft auto-save (future feature)

## Dependencies

- None — this is a bugfix to existing functionality

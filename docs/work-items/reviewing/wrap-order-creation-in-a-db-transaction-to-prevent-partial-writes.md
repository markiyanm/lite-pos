---
slug: wrap-order-creation-in-a-db-transaction-to-prevent-partial-writes
title: Wrap order creation in a DB transaction to prevent partial writes
description: handlePaymentComplete in pos/+page.svelte performs createOrder, addOrderItem
  loop, addPayment loop, completeOrder as separate SQL calls. If any step fails mid-way,
  the order is left in an inconsistent state with partial items or payments. Same
  pattern in handleSaveDraft and refund processing in orders/[id]/+page.svelte. Need
  BEGIN/COMMIT/ROLLBACK or a single atomic command.
status: reviewing
pipeline: bugfix
priority: P0
tags:
- sweep
- data-integrity
- orders
created_at: '2026-03-22T00:48:39Z'
updated_at: '2026-03-22T03:59:15Z'
rework_count: 0
artifacts:
  research: []
  design: []
  architecture: []
plan: null
dependencies: []
blocked_by: []
external_ids: {}
status_history:
- status: queued
  entered: '2026-03-22T00:48:39Z'
  exited: '2026-03-22T03:58:18Z'
- status: defining
  entered: '2026-03-22T03:58:18Z'
  exited: '2026-03-22T03:58:18Z'
- status: implementing
  entered: '2026-03-22T03:58:18Z'
  exited: '2026-03-22T03:59:15Z'
- status: reviewing
  entered: '2026-03-22T03:59:15Z'
---




## Overview

handlePaymentComplete in pos/+page.svelte performs createOrder, addOrderItem loop, addPayment loop, completeOrder as separate SQL calls. If any step fails mid-way, the order is left in an inconsistent state with partial items or payments. Same pattern in handleSaveDraft and refund processing in orders/[id]/+page.svelte. Need BEGIN/COMMIT/ROLLBACK or a single atomic command.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:48:39Z: Work item created

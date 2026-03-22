---
slug: fix-order-number-consumed-on-payment-modal-cancel
title: Fix order number consumed on payment modal cancel
description: getNextOrderNumber increments the DB counter when the payment modal opens. If user cancels, that order number is skipped permanently, creating gaps in order numbering. Fix by only generating the order number at the point of actual order creation, not when opening the modal.
status: queued
pipeline: bugfix
priority: P1
tags: ['sweep', 'orders']
created_at: 2026-03-22T00:29:37Z
updated_at: 2026-03-22T00:29:37Z
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
    entered: 2026-03-22T00:29:37Z
---

## Overview

getNextOrderNumber increments the DB counter when the payment modal opens. If user cancels, that order number is skipped permanently, creating gaps in order numbering. Fix by only generating the order number at the point of actual order creation, not when opening the modal.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:37Z: Work item created

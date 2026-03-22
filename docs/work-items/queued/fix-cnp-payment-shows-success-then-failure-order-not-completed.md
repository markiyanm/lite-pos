---
slug: fix-cnp-payment-shows-success-then-failure-order-not-completed
title: Fix CNP payment shows success then failure, order not completed
description: GitHub Issue #2. Card-not-present payment processes successfully on the gateway but the POS shows a success toast followed by a failure toast. The order is never completed. Root cause: onComplete() callback in PaymentModal is not awaited, so the parent's async order creation can fail silently after the success toast. The modal stays open and the order is not marked complete.
status: queued
pipeline: bugfix
priority: P0
tags: ['bug', 'payments', 'cnp', 'ifields', 'github-issue-2']
created_at: 2026-03-22T04:06:20Z
updated_at: 2026-03-22T04:06:20Z
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
    entered: 2026-03-22T04:06:20Z
---

## Overview

GitHub Issue #2. Card-not-present payment processes successfully on the gateway but the POS shows a success toast followed by a failure toast. The order is never completed. Root cause: onComplete() callback in PaymentModal is not awaited, so the parent's async order creation can fail silently after the success toast. The modal stays open and the order is not marked complete.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T04:06:20Z: Work item created

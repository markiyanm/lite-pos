---
slug: fix-draft-save-button-failing-with-error-toast
title: Fix draft save button failing with error toast
description: GitHub Issue #1. The Save Draft button fails with a red error toast. Root cause: getNextOrderNumber() is called outside the withTransaction() block, so if the transaction fails the order number is wasted. Additionally, any database error during order/item creation causes a generic failure.
status: queued
pipeline: bugfix
priority: P1
tags: ['bug', 'orders', 'draft', 'github-issue-1']
created_at: 2026-03-22T04:06:13Z
updated_at: 2026-03-22T04:06:13Z
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
    entered: 2026-03-22T04:06:13Z
---

## Overview

GitHub Issue #1. The Save Draft button fails with a red error toast. Root cause: getNextOrderNumber() is called outside the withTransaction() block, so if the transaction fails the order number is wasted. Additionally, any database error during order/item creation causes a generic failure.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T04:06:13Z: Work item created

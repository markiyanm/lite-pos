---
slug: fix-deviceselector-select-valueonvaluechange-type-mismatch
title: Fix DeviceSelector Select value/onValueChange type mismatch
description: shadcn-svelte Select expects value as string array but DeviceSelector
  passes a string. Card-present terminal selection in PaymentModal is broken. Fix
  by adapting to shadcn-svelte multi-value Select API.
status: reviewing
pipeline: bugfix
priority: P1
tags:
- sweep
- card-present
created_at: '2026-03-22T00:29:27Z'
updated_at: '2026-03-22T03:59:09Z'
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
  entered: '2026-03-22T00:29:27Z'
  exited: '2026-03-22T03:58:11Z'
- status: defining
  entered: '2026-03-22T03:58:11Z'
  exited: '2026-03-22T03:58:11Z'
- status: implementing
  entered: '2026-03-22T03:58:11Z'
  exited: '2026-03-22T03:59:09Z'
- status: reviewing
  entered: '2026-03-22T03:59:09Z'
---




## Overview

shadcn-svelte Select expects value as string array but DeviceSelector passes a string. Card-present terminal selection in PaymentModal is broken. Fix by adapting to shadcn-svelte multi-value Select API.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:27Z: Work item created

---
slug: fix-derived-reactivity-replace-derived-with-derivedby-across-codebase
title: Fix derived reactivity - replace derived(() =>) with derived.by() across codebase
description: 25+ locations use $derived(() => ...) instead of $derived.by(() => ...).
  In Svelte 5, $derived with an arrow function stores the function itself, not the
  computed value. Templates call the function on every access, defeating reactivity
  caching. Affects PaymentModal, OrderPanel, ProductGrid, all list pages, reports,
  settings. Should use $derived.by() for multi-statement computations.
status: reviewing
pipeline: bugfix
priority: P1
tags:
- sweep
- svelte5
- performance
created_at: '2026-03-22T00:48:55Z'
updated_at: '2026-03-22T03:59:18Z'
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
  entered: '2026-03-22T00:48:55Z'
  exited: '2026-03-22T03:58:22Z'
- status: defining
  entered: '2026-03-22T03:58:22Z'
  exited: '2026-03-22T03:58:22Z'
- status: implementing
  entered: '2026-03-22T03:58:22Z'
  exited: '2026-03-22T03:59:18Z'
- status: reviewing
  entered: '2026-03-22T03:59:18Z'
---




## Overview

25+ locations use $derived(() => ...) instead of $derived.by(() => ...). In Svelte 5, $derived with an arrow function stores the function itself, not the computed value. Templates call the function on every access, defeating reactivity caching. Affects PaymentModal, OrderPanel, ProductGrid, all list pages, reports, settings. Should use $derived.by() for multi-statement computations.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:48:55Z: Work item created

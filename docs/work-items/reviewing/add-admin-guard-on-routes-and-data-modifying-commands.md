---
slug: add-admin-guard-on-routes-and-data-modifying-commands
title: Add admin guard on routes and data-modifying commands
description: Route guard in +layout.svelte only checks isAuthenticated. Any cashier
  can navigate to /settings, /products/new, /categories, /reports. The command functions
  (createUser, updateSetting, deleteProduct, etc.) perform zero permission checks.
  Need route-level admin guards and/or command-level checks.
status: reviewing
pipeline: bugfix
priority: P1
tags:
- sweep
- security
- permissions
created_at: '2026-03-22T00:48:55Z'
updated_at: '2026-03-22T03:59:16Z'
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
  exited: '2026-03-22T03:58:18Z'
- status: defining
  entered: '2026-03-22T03:58:18Z'
  exited: '2026-03-22T03:58:19Z'
- status: implementing
  entered: '2026-03-22T03:58:19Z'
  exited: '2026-03-22T03:59:16Z'
- status: reviewing
  entered: '2026-03-22T03:59:16Z'
---




## Overview

Route guard in +layout.svelte only checks isAuthenticated. Any cashier can navigate to /settings, /products/new, /categories, /reports. The command functions (createUser, updateSetting, deleteProduct, etc.) perform zero permission checks. Need route-level admin guards and/or command-level checks.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:48:55Z: Work item created

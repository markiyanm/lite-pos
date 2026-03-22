---
slug: add-admin-guard-on-routes-and-data-modifying-commands
title: Add admin guard on routes and data-modifying commands
description: Route guard in +layout.svelte only checks isAuthenticated. Any cashier can navigate to /settings, /products/new, /categories, /reports. The command functions (createUser, updateSetting, deleteProduct, etc.) perform zero permission checks. Need route-level admin guards and/or command-level checks.
status: queued
pipeline: bugfix
priority: P1
tags: ['sweep', 'security', 'permissions']
created_at: 2026-03-22T00:48:55Z
updated_at: 2026-03-22T00:48:55Z
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
    entered: 2026-03-22T00:48:55Z
---

## Overview

Route guard in +layout.svelte only checks isAuthenticated. Any cashier can navigate to /settings, /products/new, /categories, /reports. The command functions (createUser, updateSetting, deleteProduct, etc.) perform zero permission checks. Need route-level admin guards and/or command-level checks.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:48:55Z: Work item created

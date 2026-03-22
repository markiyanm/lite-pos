---
slug: fix-typeof-import-type-assertion-error-in-settings-page
title: Fix typeof import type assertion error in settings page
description: settings/+page.svelte:481 uses 'as typeof import(printing).PrinterInfo[]' which is wrong. typeof on a module import yields the namespace type, not an interface. Change to 'as import(printing).PrinterInfo[]' or cast via the already-imported PrinterInfo type.
status: queued
pipeline: bugfix
priority: P2
tags: ['sweep', 'typescript']
created_at: 2026-03-22T00:29:51Z
updated_at: 2026-03-22T00:29:51Z
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
    entered: 2026-03-22T00:29:51Z
---

## Overview

settings/+page.svelte:481 uses 'as typeof import(printing).PrinterInfo[]' which is wrong. typeof on a module import yields the namespace type, not an interface. Change to 'as import(printing).PrinterInfo[]' or cast via the already-imported PrinterInfo type.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:51Z: Work item created

---
slug: remove-dead-code-exports-from-commands-modules
title: Remove dead code exports from commands modules
description: Several exported functions are never imported. getCategory in categories.ts, getSetting and getSettingsByGroup in settings.ts, getRefundItems in refunds.ts. Either remove them or use them where appropriate.
status: queued
pipeline: refactor
priority: P2
tags: ['sweep', 'dead-code']
created_at: 2026-03-22T00:49:13Z
updated_at: 2026-03-22T00:49:13Z
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
    entered: 2026-03-22T00:49:13Z
---

## Overview

Several exported functions are never imported. getCategory in categories.ts, getSetting and getSettingsByGroup in settings.ts, getRefundItems in refunds.ts. Either remove them or use them where appropriate.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:49:13Z: Work item created

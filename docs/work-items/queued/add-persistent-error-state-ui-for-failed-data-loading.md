---
slug: add-persistent-error-state-ui-for-failed-data-loading
title: Add persistent error state UI for failed data loading
description: All pages use onMount with try/catch that shows a brief toast on failure, but continue to render with empty data. No persistent error indicator is shown. If the DB is unreachable, users see an empty app with a disappearing toast. Add error state variables and fallback UI on each page.
status: queued
pipeline: frontend
priority: P2
tags: ['sweep', 'ux', 'error-handling']
created_at: 2026-03-22T00:49:05Z
updated_at: 2026-03-22T00:49:05Z
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
    entered: 2026-03-22T00:49:05Z
---

## Overview

All pages use onMount with try/catch that shows a brief toast on failure, but continue to render with empty data. No persistent error indicator is shown. If the DB is unreachable, users see an empty app with a disappearing toast. Add error state variables and fallback UI on each page.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:49:05Z: Work item created

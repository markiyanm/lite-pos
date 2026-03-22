---
slug: replace-deprecated-sveltecomponent-in-deviceselector
title: Replace deprecated svelte:component in DeviceSelector
description: DeviceSelector.svelte:72 uses svelte:component which is deprecated in
  Svelte 5 runes mode. Replace with the Svelte 5 dynamic component syntax. Fixes the
  svelte-check deprecation warning.
status: reviewing
pipeline: bugfix
priority: P2
tags:
- sweep
- svelte5
created_at: '2026-03-22T00:29:55Z'
updated_at: '2026-03-22T03:59:14Z'
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
  entered: '2026-03-22T00:29:55Z'
  exited: '2026-03-22T03:58:17Z'
- status: defining
  entered: '2026-03-22T03:58:17Z'
  exited: '2026-03-22T03:58:17Z'
- status: implementing
  entered: '2026-03-22T03:58:17Z'
  exited: '2026-03-22T03:59:14Z'
- status: reviewing
  entered: '2026-03-22T03:59:14Z'
---




## Overview

DeviceSelector.svelte:72 uses svelte:component which is deprecated in Svelte 5 runes mode. Replace with the Svelte 5 dynamic component syntax. Fixes the svelte-check deprecation warning.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:55Z: Work item created

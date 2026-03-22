---
slug: add-disabled-prop-to-settingtoggle-to-enforce-admin-only-settings
title: Add disabled prop to SettingToggle to enforce admin-only settings
description: SettingToggle does not accept a disabled prop so the underlying Switch
  is always enabled. Settings page passes disabled for payment method toggles and
  Sola gateway toggles, but the prop is silently ignored. Non-admin users can modify
  payment methods and gateway settings. Add disabled prop to SettingToggle and pass
  it to the Switch.
status: reviewing
pipeline: bugfix
priority: P1
tags:
- sweep
- permissions
- admin
created_at: '2026-03-22T00:29:37Z'
updated_at: '2026-03-22T03:59:11Z'
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
  entered: '2026-03-22T00:29:37Z'
  exited: '2026-03-22T03:58:13Z'
- status: defining
  entered: '2026-03-22T03:58:13Z'
  exited: '2026-03-22T03:58:13Z'
- status: implementing
  entered: '2026-03-22T03:58:13Z'
  exited: '2026-03-22T03:59:11Z'
- status: reviewing
  entered: '2026-03-22T03:59:11Z'
---




## Overview

SettingToggle does not accept a disabled prop so the underlying Switch is always enabled. Settings page passes disabled for payment method toggles and Sola gateway toggles, but the prop is silently ignored. Non-admin users can modify payment methods and gateway settings. Add disabled prop to SettingToggle and pass it to the Switch.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:37Z: Work item created

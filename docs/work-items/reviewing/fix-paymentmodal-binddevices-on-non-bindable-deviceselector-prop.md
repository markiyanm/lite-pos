---
slug: fix-paymentmodal-binddevices-on-non-bindable-deviceselector-prop
title: Fix PaymentModal bind:devices on non-bindable DeviceSelector prop
description: PaymentModal uses bind:devices on DeviceSelector, but devices is not
  declared as bindable in DeviceSelector props. The device list update binding is
  silently non-functional. Declare devices as bindable or use a callback pattern.
status: reviewing
pipeline: bugfix
priority: P1
tags:
- sweep
- card-present
created_at: '2026-03-22T00:29:37Z'
updated_at: '2026-03-22T03:59:10Z'
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
  exited: '2026-03-22T03:58:12Z'
- status: defining
  entered: '2026-03-22T03:58:12Z'
  exited: '2026-03-22T03:58:12Z'
- status: implementing
  entered: '2026-03-22T03:58:12Z'
  exited: '2026-03-22T03:59:10Z'
- status: reviewing
  entered: '2026-03-22T03:59:10Z'
---




## Overview

PaymentModal uses bind:devices on DeviceSelector, but devices is not declared as bindable in DeviceSelector props. The device list update binding is silently non-functional. Declare devices as bindable or use a callback pattern.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:37Z: Work item created

---
slug: hash-user-pins-instead-of-storing-plaintext
title: Hash user PINs instead of storing plaintext
description: auth.ts stores raw PIN in the pin_hash column. createUser saves user.pin
  directly. Anyone with SQLite file access can read all PINs. Should hash PINs before
  storage and compare hashes at login time.
status: reviewing
pipeline: bugfix
priority: P1
tags:
- sweep
- security
- auth
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
  exited: '2026-03-22T03:58:19Z'
- status: defining
  entered: '2026-03-22T03:58:19Z'
  exited: '2026-03-22T03:58:19Z'
- status: implementing
  entered: '2026-03-22T03:58:19Z'
  exited: '2026-03-22T03:59:16Z'
- status: reviewing
  entered: '2026-03-22T03:59:16Z'
---




## Overview

auth.ts stores raw PIN in the pin_hash column. createUser saves user.pin directly. Anyone with SQLite file access can read all PINs. Should hash PINs before storage and compare hashes at login time.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:48:55Z: Work item created

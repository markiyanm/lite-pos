---
slug: replace-hardcoded-aes-encryption-key-with-os-keychain
title: Replace hardcoded AES encryption key with OS keychain
description: crypto.rs has a static AES-256 key compiled into the binary as readable ASCII (LitePOS-SolaGateway-AESKey_2026!). Anyone with the binary can extract it and decrypt all Sola API keys. Should use OS keychain or derive key from a user-provided secret.
status: queued
pipeline: bugfix
priority: P1
tags: ['sweep', 'security', 'crypto']
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

crypto.rs has a static AES-256 key compiled into the binary as readable ASCII (LitePOS-SolaGateway-AESKey_2026!). Anyone with the binary can extract it and decrypt all Sola API keys. Should use OS keychain or derive key from a user-provided secret.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:48:55Z: Work item created

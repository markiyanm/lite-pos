---
slug: fix-escpos-column-layout-for-non-ascii-characters
title: Fix ESC/POS column layout for non-ASCII characters
description: escpos_two_col and escpos_text use .len() which is byte length, not character count. Multi-byte UTF-8 characters (accented letters, international scripts) in store names or addresses will produce misaligned receipt columns. Use char_count or unicode-width crate for correct column calculations.
status: queued
pipeline: bugfix
priority: P2
tags: ['sweep', 'printing', 'i18n']
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

escpos_two_col and escpos_text use .len() which is byte length, not character count. Multi-byte UTF-8 characters (accented letters, international scripts) in store names or addresses will produce misaligned receipt columns. Use char_count or unicode-width crate for correct column calculations.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:51Z: Work item created

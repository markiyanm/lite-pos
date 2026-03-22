---
slug: remove-or-implement-printhtml-windows-stub-in-printingrs
title: Remove or implement print_html Windows stub in printing.rs
description: print_html_windows unconditionally returns an error. The command is registered
  and exported to the frontend. On Windows (the primary platform), any call to printHtml
  will fail. Either remove the command and its frontend export, or implement it properly.
status: reviewing
pipeline: bugfix
priority: P2
tags:
- sweep
- printing
created_at: '2026-03-22T00:29:51Z'
updated_at: '2026-03-22T03:59:13Z'
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
  entered: '2026-03-22T00:29:51Z'
  exited: '2026-03-22T03:58:15Z'
- status: defining
  entered: '2026-03-22T03:58:15Z'
  exited: '2026-03-22T03:58:15Z'
- status: implementing
  entered: '2026-03-22T03:58:15Z'
  exited: '2026-03-22T03:59:13Z'
- status: reviewing
  entered: '2026-03-22T03:59:13Z'
---




## Overview

print_html_windows unconditionally returns an error. The command is registered and exported to the frontend. On Windows (the primary platform), any call to printHtml will fail. Either remove the command and its frontend export, or implement it properly.

## Acceptance Criteria

- [ ] (Define acceptance criteria)

## Notes

- 2026-03-22T00:29:51Z: Work item created

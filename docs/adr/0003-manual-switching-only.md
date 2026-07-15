# ADR 0003: Manual Switching Only

- Status: Accepted for project scope
- Date: 2026-07-15

## Context

Multi-account tools often add quota polling, account scoring, and automatic rotation. Those features can turn identity separation into rate-limit circumvention and increase account-enforcement risk.

## Decision

Every profile change requires an explicit user command. Do not inspect quota or retry another account automatically. Do not expose provider access through an API.

## Consequences

- Narrower, safer product with simpler architecture.
- No transparent failover.
- Contributions implementing quota pooling, proxying, or fingerprint spoofing are rejected as out of scope.

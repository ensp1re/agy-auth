# ADR 0001: Use a Rust Workspace

- Status: Proposed
- Date: 2026-07-15

## Context

The application handles high-value local secrets, performs cross-platform atomic file changes, and should install without a language runtime.

## Decision

Use stable Rust with separate CLI, application, domain, storage, and provider crates. Keep the execution path synchronous.

## Consequences

- Strong compile-time boundaries and single-binary releases.
- Higher learning curve and cross-platform keyring build complexity.
- More disciplined secret wrapper types and recovery guards.
- Go remains the fallback if maintainer expertise or keyring portability blocks Rust delivery.

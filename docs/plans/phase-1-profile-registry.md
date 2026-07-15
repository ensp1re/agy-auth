# Phase 1 profile registry execution plan

## Objective

Implement the versioned, non-secret registry described in `docs/05-repository-structure.md` without
adding credential capture, provider activation, or user-facing profile commands.

## Constraints

- Domain types and invariants must not depend on filesystem or CLI libraries.
- The registry stores metadata only; credential payloads and full account emails are forbidden.
- Mutations require an exclusive lock and same-directory atomic replacement.
- Unsupported versions, unsafe file types, and malformed data fail closed.
- All test fixtures are obviously synthetic.

## Steps

1. Define profile identifiers, normalized names, provider/storage/status enums, and registry errors.
2. Define schema v1 serialization and validation at the storage boundary.
3. Implement locked reads and atomic writes with explicit durability and permission handling.
4. Add migration dispatch with v1 as the only accepted current version.
5. Test round trips, duplicate names, malformed/unsupported data, locking, and atomic replacement.
6. Run the complete project and dependency-policy gates, then review secret-bearing field names.

## Completion evidence

Record exact commands, revision, PR, and merge commit in `.harness/state/work.json`. Hosted CI remains
deferred until account usage is restored; do not represent unrun platforms as passing.

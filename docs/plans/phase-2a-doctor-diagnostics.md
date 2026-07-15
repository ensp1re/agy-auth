# Phase 2A Plan: Doctor Diagnostics Vertical Slice

## Objective

Deliver a truthful `agy-auth doctor` command that diagnoses the official executable, project-owned
registry, and supported capabilities without accessing Antigravity authentication state.

## Scope

- discover `agy` from an explicit safe path or caller-supplied `PATH`;
- run only the bounded `agy --version` probe;
- load and validate only the project-owned non-secret registry;
- emit deterministic human-readable and JSON output;
- report profile switching as unsupported and authentication mutation as disabled;
- make `--repair` a safe no-op while no approved repair action exists.

## Exclusions

No interactive `agy` launch, login/logout, account detection, keyring access, credential files,
environment dumps, model requests, state mutation, or provider repair.

## Acceptance

- installed, missing, invalid, and failed-version client states have stable categories;
- healthy, absent, corrupt, unsafe, and unsupported registry states fail closed;
- JSON has a versioned stable shape and contains no paths or identity data;
- exit codes follow the CLI contract: `0` healthy, `4` client unavailable, `8` registry unsafe;
- all workspace, dependency, secret, harness, and manual CLI checks pass locally.

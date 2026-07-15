# Phase 1 Gemini isolated-home adapter execution plan

## Objective

Implement the Gemini CLI provider boundary for isolated profile homes while keeping real execution
disabled until `GEMINI_CLI_HOME` credential isolation is verified on supported platforms.

## Constraints

- Do not read, parse, copy, or infer Gemini credential state.
- Do not launch login, make model requests, or mutate the official default home.
- Derive managed homes only from a trusted data root and immutable `ProfileId`.
- Child environment changes are an allowlist: set `GEMINI_CLI_HOME`; remove only reviewed conflicting
  authentication variables; never capture or log their values.
- Synthetic capability evidence may enable fake-client tests but must not be constructible as real
  production evidence accidentally.

## Steps

1. Add process infrastructure to the Gemini provider dependency boundary.
2. Define discovery and isolated-home capability evidence types.
3. Derive and create owner-only managed profile homes without following links.
4. Build direct executable/argv/environment execution plans.
5. Fail closed when isolation evidence is absent or stale.
6. Test fake clients, path containment, permissions, argv, and child-only environment behavior.
7. Run all local gates and open a review PR; record real-client verification as not run.

## Research still required

Install a supported Gemini CLI version in an isolated environment and verify that every credential and
keyring lookup respects `GEMINI_CLI_HOME` on Linux, macOS, and Windows. Record versioned evidence in an
ADR or compatibility note before enabling production profile execution.

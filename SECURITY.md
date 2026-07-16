# Security Policy

`agy-auth` handles local authentication state for the official Google Antigravity CLI. Vulnerability
reports involving credential disclosure, filesystem mutation, permission bypass, or transaction
recovery are treated seriously.

## Supported versions

| Version | Supported |
|---|---|
| `main` | Yes |
| Latest published pre-release | Best effort |
| Older pre-releases and development snapshots | No |

The current authentication-state contract is separately limited to verified Antigravity CLI
versions and platforms. Run `agy-auth doctor` to determine whether the installed client/environment
is supported.

## Reporting a vulnerability

Do not open a public issue for a suspected security vulnerability.

Use [GitHub Private Vulnerability Reporting](https://github.com/ensp1re/agy-auth/security/advisories/new)
for this repository:

1. Open the private reporting form.
2. Describe the affected `agy-auth` revision, `agy` version, platform, and environment.
3. Explain the impact and required attacker capabilities.
4. Provide minimal reproduction steps using synthetic credentials and disposable paths.
5. Include suggested containment or remediation when known.
6. Wait for maintainer coordination before public disclosure.

If GitHub private reporting is unavailable, open a public issue containing only the statement that
you need a private security contact. Do not include vulnerability details or sensitive material.

Never submit:

- reusable access, refresh, or ID tokens;
- Google passwords, OAuth authorization codes, state, verifier, or private URLs;
- full account emails;
- credential files, keyring exports, environment dumps, or unredacted logs;
- private customer, employer, or workspace data.

If sensitive evidence is essential, first describe its type and obtain a private transfer method
from the maintainers.

## What to include

A useful report contains:

- vulnerability class and affected component;
- affected commit, release, operating system, architecture, and `agy` version;
- reproducible preconditions and steps;
- observed and expected security behavior;
- impact on confidentiality, integrity, or availability;
- whether exploitation requires the same OS user, another local user, or remote access;
- minimal synthetic proof of concept;
- temporary mitigation, if available.

## High-impact vulnerability classes

Examples include:

- credential or full-account-identity disclosure;
- unsafe ownership or permission acceptance;
- symlink or hardlink overwrite;
- mutation outside project-owned or approved official-client paths;
- rollback failure or transaction corruption;
- secret logging, panic output, or terminal leakage;
- profile confusion or cross-account credential capture;
- version/capability-gate bypass;
- argument injection or shell evaluation;
- recovery deleting a ready profile or unrelated state.

## Response process

The maintainers aim to:

- acknowledge a report within 7 days;
- provide an initial assessment within 14 days;
- coordinate remediation and disclosure within 90 days when practical;
- credit reporters who request attribution, unless doing so would expose private information.

Timelines may change with severity, project maturity, upstream dependencies, or the need to coordinate
with Google. Please do not publicly disclose the issue before a fix or agreed mitigation is
available.

## Security boundaries and limitations

`agy-auth` is designed to reduce additional local risk, not to protect against a fully compromised
same-user session or root/administrator access.

The project does not:

- implement Google OAuth or model/backend protocols;
- query private usage, quota, entitlement, or account APIs;
- automatically rotate or load-balance accounts;
- synchronize or share credentials;
- claim that reverse-engineered Antigravity behavior is supported by Google.

See [docs/08-security-policy.md](docs/08-security-policy.md) for the complete project security
contract.

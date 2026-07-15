# Security policy

This project is pre-release. No version currently receives security fixes as a supported release.

Do not report vulnerabilities through a public issue when they could expose credentials, bypass
permission checks, overwrite files through links, break rollback, or leak secrets. Use GitHub's
private vulnerability reporting for this repository. Do not include real credential files, tokens,
OAuth URLs, full email addresses, private logs, or environment dumps in a report.

High-impact areas include credential disclosure, unsafe ownership or permissions, symlink/hardlink
overwrite, secret logging, transaction corruption, and failed rollback. Reports should contain a
minimal synthetic reproduction, affected revision and platform, impact, and suggested containment.

The maintainers target an initial response within 7 days and coordinated remediation within 90 days,
subject to project maturity and severity. Public disclosure should wait until a fix or agreed
mitigation is available.

# Security Policy

## Reporting

Do not open a public issue containing a vulnerability, leaked credential, or
private data. Use GitHub's private vulnerability reporting for this repository.

If a secret has entered a commit, treat it as compromised immediately:

1. revoke or rotate it before changing Git history;
2. report the affected service and commit privately;
3. coordinate any history rewrite with maintainers;
4. re-run the full-history secret scan before pushing repaired history.

Deleting a secret from the latest revision does not remove it from Git history.

## Supported code

Until Merra begins publishing releases, security fixes target `main`. The
project does not currently promise support for old development snapshots.

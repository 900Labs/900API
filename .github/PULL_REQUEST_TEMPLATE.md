## What changed

Describe the user problem and the focused solution.

## Verification

- [ ] `./scripts/verify-local.sh` passes
- [ ] New or changed behavior has focused tests
- [ ] Error and recovery paths were tested
- [ ] User-facing behavior and public documentation agree

List any additional manual checks and their results.

## Local-first review

- [ ] No telemetry, hosted account, or mandatory cloud service was added
- [ ] Low-bandwidth and older-hardware impact was considered
- [ ] No credential, private URL, personal path, local database, or generated build output is included
- [ ] Portable collection changes use `api900-core` across desktop, Git Sync, and CLI

## Interface changes

Add screenshots for visible changes. Remove private endpoints, credentials, personal paths, and customer data first.

## Remaining limits

State any behavior that remains unsupported or could not be verified.

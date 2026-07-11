# Public Releases

900API follows semantic versioning. The first public release prepared by this repository is `v0.2.0`.

## Before Creating a Tag

1. Confirm the worktree contains only intended release changes.
2. Run `./scripts/verify-local.sh`.
3. Run `npm audit --audit-level=high`.
4. Run `cargo audit` and review every warning.
5. Build and open a native bundle on the current platform.
6. Confirm the version is `0.2.0` in npm, both Rust crates, the shared core crate, Tauri configuration, and the changelog.
7. Review README, support, security, contribution, issue, and pull request text as it appears on GitHub.
8. Confirm repository About text and topics match `docs/GITHUB_METADATA.md`.
9. Confirm CI passes on the release commit.
10. Review commit authors, committer identities, commit messages, branches, tags, and remotes separately from the file privacy scan.

Do not create the tag if any required check fails.

## Tag Workflow

Tags matching `v*` trigger `.github/workflows/release.yml`. Before any asset is published, the workflow requires the tag to equal `v` plus the version in Tauri configuration, `package.json`, and every 900API Cargo package. A malformed or mismatched tag stops the workflow. The workflow then uses the official `tauri-apps/tauri-action@v1` matrix pattern to build:

- macOS arm64
- macOS x86_64
- Ubuntu 22.04 x86_64
- Windows x86_64

When all platform jobs finish, a final job downloads the release assets, creates `SHA256SUMS.txt`, and uploads it to the release.

Create the annotated tag only from the reviewed release commit:

```bash
git tag -a v0.2.0 -m "900API 0.2.0"
git push origin v0.2.0
```

The control tower owns these commands and all GitHub settings changes.

## Verify a Download

Download the artifact and `SHA256SUMS.txt` from the same release. On macOS or Linux:

```bash
shasum -a 256 <artifact>
```

On Windows PowerShell:

```powershell
Get-FileHash <artifact> -Algorithm SHA256
```

Compare the result with the matching checksum entry.

## Signing Status

The initial automated release artifacts are unsigned unless repository signing secrets are added before the tag is pushed.

- macOS builds use an ad-hoc signing identity so Apple Silicon bundles retain valid bundle structure after download. Ad-hoc signing is not Developer ID signing and is not notarization.
- Windows builds are not Authenticode-signed without a code-signing certificate.
- Linux packages are not distribution-repository signed.

Document these limits in the release notes. Do not describe unsigned artifacts as trusted or notarized. Code signing and notarization are a later release improvement.

## Repository Publication

Before changing visibility to public, verify that the release commit and tag contain no private paths, credentials, personal emails, private working notes, or local build artifacts. Set the GitHub About description, website, and topics from `docs/GITHUB_METADATA.md`. Enable issues and discussions only if maintainers are ready to respond under `SUPPORT.md`.

The privacy script scans file content that is tracked or eligible for commit. It does not inspect Git object metadata or history. The control tower must separately review commit authors, committer identities, commit messages, branches, tags, and remotes. This release preparation does not rewrite history; any metadata remediation belongs to the publication owner.

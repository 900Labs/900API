# Public Releases

900API follows semantic versioning. The current maintenance release is `v0.2.1`.

## Before Creating a Tag

1. Confirm the worktree contains only intended release changes.
2. Run `./scripts/verify-local.sh`.
3. Run `npm audit --audit-level=high`.
4. Run `cargo audit` and review every warning.
5. Build and open a native bundle on the current platform.
6. Confirm the version is `0.2.1` in npm and its lockfile, all Rust packages and their lockfile entries, Tauri configuration, visible app fallbacks, and the changelog.
7. Review README, support, security, contribution, issue, and pull request text as it appears on GitHub.
8. Confirm repository About text and topics match `docs/GITHUB_METADATA.md`.
9. Confirm CI passes on the release commit.
10. Review commit authors, committer identities, commit messages, branches, tags, and remotes separately from the file privacy scan.

Do not create the tag if any required check fails.

## Tag Workflow

Tags matching `v*` trigger `.github/workflows/release.yml`. The workflow can also be started manually for an existing tag. The quality job checks out only the explicit `refs/tags/<tag>` namespace, verifies the version metadata, resolves the tag to one immutable commit SHA, and exports that SHA. Build and checksum jobs check out only the exported SHA and never resolve the selected tag text as their build ref. Immediately before each tag-addressed release operation, the job refreshes the tag from origin and fails if it no longer resolves to the exported SHA.

A missing, malformed, moved, or version-mismatched tag stops the workflow. The workflow uses Rust 1.97.0 and the official `tauri-apps/tauri-action@v1` matrix pattern to build:

- macOS arm64
- macOS x86_64
- Ubuntu 22.04 x86_64
- Windows x86_64

When all platform jobs finish, a final job downloads the release assets and requires exactly one nonempty package matching each canonical 900API filename:

- `900API_<version>_aarch64.dmg`
- `900API_<version>_x64.dmg`
- `900API_<version>_amd64.AppImage`
- `900API_<version>_amd64.deb`
- `900API-<version>-<rpm release>.x86_64.rpm`
- `900API_<version>_x64_<locale>.msi`
- `900API_<version>_x64-setup.exe`

The only additional regular files allowed are nonempty current-version `aarch64.app.tar.gz` and `x64.app.tar.gz` archives with the `900API_` prefix, plus an existing `SHA256SUMS.txt` from a rerun. The checksum job ignores and regenerates that file. Stale, duplicate, differently named, and unexpected regular assets fail the workflow. Only after validation passes does the workflow create and upload the new `SHA256SUMS.txt`.

This validates the release artifact set and confirms that the expected package files were generated. It does not install them and does not test fresh-install or upgrade behavior. Those platform smoke tests remain separate release work.

Create the annotated tag only from the reviewed release commit:

```bash
git tag -a v0.2.1 -m "900API 0.2.1"
git push origin v0.2.1
```

The control tower owns these commands and all GitHub settings changes.

To rerun an existing tag, open the Release workflow in GitHub Actions, choose **Run workflow**, and enter the existing tag such as `v0.2.1`. The quality job resolves that explicit tag once, and every later job remains bound to the resulting commit SHA. The workflow does not create or move a tag.

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

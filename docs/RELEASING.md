# Releasing

## One-time setup

Create these repository secrets (environment secrets once the repository is
public and on a plan that has them):

| Secret | Contents |
|---|---|
| `APPLE_CERTIFICATE` | Base64 of the Developer ID Application `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | That file's password |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: NAME (TEAMID)` |
| `APPLE_API_ISSUER` | App Store Connect issuer ID |
| `APPLE_API_KEY` | App Store Connect key ID |
| `APPLE_API_PRIVATE_KEY` | Contents of `AuthKey_<id>.p8` |

The updater is not enabled yet; its keys come with the first public release.

## Cutting a release

```sh
node scripts/bump-version.ts 0.2.0     # Cargo.toml, tauri.conf.json, package.json
pnpm install && cargo update -w        # refresh lockfiles for the new version
git commit -am "chore(release): 0.2.0"
git tag v0.2.0 && git push origin develop v0.2.0
```

`.github/workflows/release.yml` then:

1. creates a draft GitHub release for the tag;
2. builds on `macos-15` (universal, signed, notarized), `windows-2025` (x64)
   and `ubuntu-22.04` (x64), uploading every bundle to the draft;
3. writes `SHA256SUMS` and, when the repository is public, attests build
   provenance over it.

Install each asset from the draft on a clean machine, run
[TESTING.md](TESTING.md), then publish the draft by hand.

Test builds without a release: run the workflow manually (Actions, release,
Run workflow); bundles arrive as workflow artifacts.

## While the repository is private

- CI runs Linux on every change; macOS and Windows jobs run on manual dispatch
  (their minutes count 10x and 2x).
- Windows builds are unsigned until SignPath signing is set up after going
  public.
- Build provenance attestations switch on automatically once public.

## Going public

1. Create a `release` environment with a required reviewer; move the secrets
   into it; add `environment: release` to the build job.
2. Add rulesets: only maintainers create `v*` tags; `develop` requires pull
   requests and green CI.
3. Apply to SignPath Foundation and add its signing step for Windows.
4. Add an `ubuntu-22.04-arm` build.
5. Enable the updater with its own signing key.
6. Tag a fresh release so the first public one carries attestations.

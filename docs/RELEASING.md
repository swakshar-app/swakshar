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
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of the updater private key file |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Its password |

The updater key signs every update; the app carries its public key. Create
it once, on the owner's Mac:

```sh
pnpm tauri signer generate -w ~/.tauri/swakshar-updater.key
```

Store the private key file and its password in the password manager and in
the two secrets above; losing them strands every install on its current
version. Paste the public key (`~/.tauri/swakshar-updater.key.pub`) into
`plugins.updater.pubkey` in `app/src-tauri/tauri.conf.json` and set
`bundle.createUpdaterArtifacts` to `true`. Until then the app reports
"Updates are not set up in this build".

## Cutting a release

```sh
node scripts/bump-version.ts 0.2.0     # Cargo.toml, tauri.conf.json, package.json
pnpm install && cargo update -w        # refresh lockfiles for the new version
git commit -am "chore(release): 0.2.0"
git tag v0.2.0 && git push origin develop v0.2.0
```

Before the first release, prove updates end to end: tag `v0.1.0-rc.1`,
install it, publish its draft, tag and publish `v0.1.0-rc.2`, and watch rc.1
offer, download and restart into rc.2.

`.github/workflows/release.yml` then:

1. creates a draft GitHub release for the tag;
2. builds on `macos-15` (universal, signed, notarized), `windows-2025` (x64)
   and `ubuntu-22.04` (x64), uploading every bundle to the draft. On macOS
   the app is notarized and stapled by Tauri, then the disk image is
   notarized and stapled too and replaces the first upload;
3. writes `SHA256SUMS` and, when the repository is public, attests build
   provenance over it.

Before installing, check the macOS image the way Gatekeeper will:

```sh
spctl -a -t open --context context:primary-signature -v Swakshar_*_universal.dmg
xcrun stapler validate Swakshar_*_universal.dmg
```

Both must pass (`accepted`, `source=Notarized Developer ID`). Then install
each asset from the draft on a clean machine, run [TESTING.md](TESTING.md),
and publish the draft by hand.

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

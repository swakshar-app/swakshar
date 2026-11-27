# Decisions

Newest first. Preserve earlier entries in the linked archive when this ledger
approaches 300 lines.

## Seal the assembled Python framework before transfer

`2026-11-27-11-15-00-UTC`

**Decision.** After PyInstaller collection and the runtime copy finish, sign the
completed Python.framework as a bundle with the already-approved Developer ID,
secure timestamp and hardened runtime. Then enforce the strict signature checks
before archiving and after transfer. Preserve symlinks during final app copying.

**Reasoning.** The new native gate in run 45988604947 caught the arm64 framework
before Tauri: codesign reported that code has no resources but its signature
requires them. PyInstaller had signed the collected binary before assembling
the framework and its Info.plist, leaving no complete framework resource seal.
Sign the assembled bundle rather than weakening verification. This complements
the symlink-preservation fix and reuses the owner's existing signing approval;
no keys, credentials or entitlements are changed.

## Preserve signed Python framework links in the Mac installer

`2026-11-27-11-13-00-UTC`

**Decision.** On macOS, replace generic resource copying with Tauri's custom
files mapping to the same Contents/Resources locations. Verify the Python
framework's required symlinks and strict code signatures on native CI builders
before archiving and after restoring both runtimes. Keep the existing signing
identity, entitlements and mandatory app/disk-image notarization gates.

**Reasoning.** After the owner resolved Apple's account agreement, notarization
submission 79266ccd-2597-41aa-9138-e1400aaad405 rejected both Python framework
binaries as invalid. The pinned Tauri CLI's generic resource copier copies each
file with fs::copy, dereferencing links. Its custom-directory copier explicitly
recreates symlinks. PyInstaller requires those links to survive distribution.
Use the preserving copier rather than altering credentials or bypassing checks.
CI will establish the original and restored signatures before final notarization.

## Gate publication on the final tagged CI build

`2026-11-27-13-26-00-IST`

**Decision.** Once source CI is green, validate the corrected Windows archive
tool in the unpublished v0.1.10 candidate's full release workflow. Publish only
after every native runtime and installer, Apple notarization, checksums and
provenance pass. This supersedes requiring a complete additional manual build
before the candidate tag; it does not relax any publication gate.

**Reasoning.** Manual CI has already verified all four native PDF runtimes,
Linux installers and universal Mac compilation/signing. The owner has resolved
Apple's pending account agreement. The final workflow must package and notarize
the exact tagged source in all cases; use that draft build to verify the Windows
native tar correction rather than producing the same installers twice first.

## Select Windows' native archive tool explicitly

`2026-11-27-13-21-00-IST`

**Decision.** Extract PDF runtimes with System32's native `tar.exe` on Windows.
Keep the relative archive filename and use the platform tar on macOS and Linux.
This supersedes relying on whichever tar Git Bash puts first on PATH.

**Reasoning.** The first extraction fix removed GNU tar's remote-host parsing
of the archive path, but CI then showed that the same MSYS executable also
rejects the absolute native Windows destination path. Selecting the operating
system's archive tool handles drive letters and backslashes consistently with
the successful runtime creation job, which ran outside Git Bash.

## Reserve a fresh tag for the verified acceptance build

`2026-11-27-13-06-00-IST`

**Decision.** Advance the app and workspace packages to 0.1.10. Preserve failed
release tags and unpublished drafts for 0.1.8 and 0.1.9. Tag the new version only
after source CI and the corrected manual packaging workflow pass, then publish
the tagged build as a prerelease after notarization, checksums and provenance.

**Reasoning.** The owner will test from a release download. A complete changelog
entry must describe both signing features because earlier versions were never
published. Keep v0.1.7 as the stable updater release while acceptance is pending.

## Extract runtime archives without Windows drive-letter ambiguity

`2026-11-27-13-02-00-IST`

**Decision.** Run runtime extraction from the downloaded artifact directory and
pass its relative archive filename to tar. Keep the destination absolute.

**Reasoning.** All four frozen runtimes passed in manual packaging CI. The
Windows app job then invoked Git Bash's GNU tar, which parsed the colon in an
absolute `D:\\...` archive path as a remote host separator. A relative archive
filename works with both GNU tar and the native tar tools on every runner.

## Keep isolated crypto build tools on the subprocess path

`2026-11-27-12-51-00-IST`

**Decision.** Prepend the private Python environment's executable directory to
the Intel cryptography source builder's PATH. Keep installations, verification
and release builds in CI, as the owner requested. Exercise packaging through a
manual workflow run before creating another immutable release tag.

**Reasoning.** The v0.1.9 runtime jobs passed on Apple Silicon, Windows and Linux.
Intel compiled static OpenSSL but failed because Maturin's Python entrypoint
launches the `maturin` executable by name. Invoking pip through the private
Python executable does not activate that environment's PATH. Its pinned wheel
was installed correctly; expose that environment's tools to its subprocesses.
The failed release remains an unpublished draft.

## Pin a Python runtime available on every release builder

`2026-11-27-12-30-00-IST`

**Decision.** Package Python 3.14.10 and regenerate the runtime's age-gated
wheel lock for Python 3.14. GitHub's runtime catalog provides native macOS arm64
and x64, Windows x64 and Ubuntu 22.04 x64 binaries. Include its exact upstream
license. Version 3.14.10 was tagged on 2026-10-25 and its runner distributions
were published on 2026-10-26, both older than seven days.

**Intel crypto.** Cryptography 50.0.2 no longer publishes Intel Mac wheels. Build
its original, hash-locked source on that runner using pinned Maturin and upstream's
enforced Rust lockfile, with static OpenSSL 4.0.3 from its dated source/hash lock.
Keep current cryptography rather than downgrading it for Intel. Carry the OpenSSL
license and provenance in the frozen runtime; users need no external OpenSSL.

**Reasoning.** Source CI passed, but the first packaging run could not obtain
Python 3.12.16 on macOS or Windows. Source-release availability alone did not
establish runner binary availability. Check the actual binary catalog and run
the frozen CLI before producing installers. Keep the failed v0.1.8 tag immutable
and create v0.1.9 for the corrected build after its CI passes. The original
emBridge ZIP is now hosted as a hash-checked dependency release, and its HTTPS
URL is configured in the repository variable used by native builds.

## Verify CI and publish a downloadable acceptance build

`2026-11-27-12-19-19-IST`

**Decision.** The owner authorized fixing failing CI and creating a release once
checks pass, with real-token and portal acceptance performed from the downloaded
build. Run the source gates and signed packaging workflow. Publish the resulting
acceptance build as a prerelease; leave the existing stable updater release in
place until acceptance. This supersedes the earlier no-build/no-release scope.

**Reasoning.** The first CI run found a removed PDF.js 6 rectangle API and lint
issues, plus a Tauri core/transitive version mismatch. Use viewport point
transforms, keyed editor/page state and current compatible Tauri 2.12.7 with
tauri-build 2.7.6. Update tauri-plugin-updater to 2.14.1 so its ZIP dependency can
resolve to the patched major version instead of vulnerable ZIP 4.

**Age approval.** The owner explicitly approved url 2.5.9 and cookie 0.18.3 as
two security patch exceptions to the seven-day dependency age on 2026-11-27.
These address RUSTSEC-2026-0592 and RUSTSEC-2026-0594. URL 2.5.9 requires
form_urlencoded ^1.2.3, so that mandatory dependency accompanies the approved
URL patch; Cargo rejects the older 1.2.2. Keep unrelated dependencies under the
seven-day rule. No security advisories are suppressed.

## Consolidate Dependabot fixes with the signing features

`2026-11-27-12-08-42-IST`

**Decision.** Include the open Cargo, pnpm and GitHub Actions dependency groups
in the same PR, as the owner requested. Pin Rustls 0.23.46, Vite 8.3.4, Vitest
5.0.5 and Happy DOM 21.0.4, alongside the other grouped updates. Refresh the
Cargo and pnpm lockfiles with their seven-day publication gates. Use current
GitHub Action releases eligible under the age gate with full commit pins,
including the new runtime jobs. cargo-deny-action stays at 2.1.3 because 2.1.4
was published on 2026-11-24 and cannot yet meet the seven-day minimum.

**Reasoning.** These versions cover the repository's 20 open Dependabot security
alerts as inspected on 2026-11-27. The proposed Happy DOM 21.0.3 update still
falls below the advisory's patched 21.0.4, so use the latter. Rustls and Tokio
remain within the selected architecture and their existing major versions.
Release metadata and minimum Rust versions support the pinned Rust toolchain.
Alerts remain open until GitHub scans the merged dependency graph. Leave the
existing unrelated advisory policy unchanged. Version 0.1.8 documents the
combined feature and dependency change; it does not authorize a release.

## Bundle the signing runtime and the permitted portal companion

`2026-11-27-11-56-32-IST`

**Decision.** Add Income Tax portal companion controls on macOS and local PDF
signing with selectable page and drawn placement. Ship PDF.js assets, pyHanko,
PKCS#11 bindings and Python inside platform installers. Use only the bundled
PDF executable. Keep the existing Rust core and TypeScript UI.

**Reasoning.** The owner requested these two signing workflows and then required
runtime dependencies to ship in the build. PDF.js handles viewport rotation and
crop coordinates; pyHanko supplies the PDF incremental-signature implementation,
without changing the GST CMS protocol. A frozen native runtime avoids requiring
end users to install Python or command-line tools. Universal Mac packages need
both native CPU runtimes. Exact version/hash locks and the seven-day age rule
also apply to these packaging dependencies.

**Signing approval.** The owner explicitly approved reusing the existing macOS
CI signing credentials for the bundled runtime on 2026-11-27. Apple's pinned
certificate-import action provides a temporary runner keychain; PyInstaller
signs the helper and its native libraries using the existing Developer ID and
entitlements before Tauri signs and notarizes the surrounding app. No credentials
were read, changed or used during implementation, and no build was launched.

**Vendor permission.** The owner reports eMudhra replied by email that this is
permitted while Swakshar is not monetized. Package its unmodified macOS installer
under that reported condition; keep the email private and revisit before any
monetization. This records the owner's account, not an independently verified
public redistribution license. emBridge remains subject to its vendor terms.
Its binaries are build inputs, ignored by git, not Apache-2.0 source. Version
3.2.7.0 is pinned because the latest vendor download is younger than seven days.
The offline installer still runs the vendor's native, administrator-approved
installation; Swakshar manages the resulting service rather than claiming native
Income Tax protocol support. Hardware drivers and Apple's Rosetta remain
manufacturer or OS components.

## Keep PDF approval local, bounded and separate from portal signing

`2026-11-27-11-56-31-IST`

**Decision.** Sign the exact native-picker-selected byte snapshot. Accept page
and rectangle plus an opaque token/certificate selection, not arbitrary file
paths or executable names from the webview. Share the existing single signing
slot with GST, refuse locked certificates, require fresh lockout flags after a
failure, and never retry automatically. Pass the PIN only in the child process's
environment, never in arguments, profile files or logs. Cancel the process on
request, editor exit, app quit or deadline. Save a new file with atomic no-replace
publication; clean normal staging paths. Reject PIN formats the upstream CLI
would trim, and include only the selected certificate in its signature payload.

**Reasoning.** Preview and signing must cover identical bytes, another application
must not lose a file created after the Save dialog, and a wrong PIN must not
silently consume more attempts. Token identity is resolved again after the Save
dialog. Brief exposure to same-user process inspection is recorded in the threat
model. The first PDF version signs offline without TSA/LTV network calls and
creates a new field. Additional timestamping or existing-field workflows need
separate implementation. emBridge keeps its own PIN and approval UI. Owner
verification remains authoritative: no tests, app launch, browser or installer
build were run. See [build and verification instructions](docs/PDF_SIGNING.md).

## Earlier decisions

- [Prior ledger](decisions/2026-11-27-prior.md): the unchanged decisions through
  2026-10-02 and links to the earlier dated archives.

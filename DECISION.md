# Decisions

Newest first. Preserve earlier entries in the linked archive when this ledger
approaches 300 lines.

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

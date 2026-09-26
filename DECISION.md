# Decisions

Newest first. Each entry has a title, an IST timestamp, the decision and the
reasoning. Supersede an entry with a new one; never rewrite an old entry. Keep
this file under 300 lines by moving older entries into `decisions/` with an
index.

## Token drivers come from their makers; Swakshar guides the install

`2026-09-26-20-36-00-IST`

**Decision.** Swakshar does not ship vendor PKCS#11 drivers. It lists USB
devices with `nusb` (pure Rust, no vendor code), recognises Feitian,
Hypersecu and SafeNet tokens by their public vendor IDs and any other token
by the smart card interface class, and tells the user which driver is
missing, built for the other processor, failing, or not seeing the token.
Installed drivers are picked up on the next scan. Drivers in unusual places
are added with a native file picker that `tauri-plugin-dialog` opens from
Rust, so no window gains a dialog permission. Bundling OpenSC, which is open
source and has its own ePass2003 support, is the route to a token that works
with nothing installed; it waits until OpenSC is verified with real
ePass2003 and HYP2003 tokens.

**Reasoning.** Vendor drivers are proprietary and redistributing them needs
each vendor's permission. The clean-room rule bars vendor binaries from the
repository. Anything inside the bundle is signed with Swakshar's Developer
ID and notarized as ours, which would vouch for closed code nobody here can
audit, and vendors ship driver fixes for new macOS releases that a bundled
copy would miss. Naming the token and the exact driver it needs removes the
guesswork, which is where users got stuck.

## cargo-deny fails on what we can act on

`2026-09-26-15-23-00-IST`

**Decision.** `deny.toml` sets `unmaintained = "workspace"`, so unmaintained
advisories fail only for crates we depend on directly, and
`allow-wildcard-paths = true`, so our private crates may reach each other by
path. Vulnerability, yanked, licence and source checks are unchanged and
still cover the whole graph. Unused licences are dropped from the allow list.

**Reasoning.** The first CI runs failed on `proc-macro-error` (pulled in by
gtk-rs on Linux) and five `unic-*` crates (pulled in by Tauri's URL
patterns). Neither can be replaced from this repository, so a failure there
blocks every change without a fix to make; the advisories stay visible in
upstream updates. Path dependencies between `publish = false` crates are
never resolved from a registry, so the wildcard ban has nothing to protect
there, and cargo-deny limits this allowance to private crates.

## Apache-2.0 is the only licence

`2026-09-26-01-44-00-IST`

**Decision.** Swakshar is licensed under the Apache License 2.0 alone.
`LICENSE` holds the text and every manifest says `Apache-2.0`. This supersedes
the licence in "Owner choices for the build"; the rest of that entry stands.

**Reasoning.** Chosen by the owner on 2026-09-26. Apache-2.0 keeps what the
dual licence was for: an explicit patent grant, and an OSI-approved licence
that qualifies for SignPath's free signing. One licence is simpler to state,
to comply with, and to accept contributions under.

## Signatures are pinned byte for byte to the Python signer

`2026-09-26-01-43-00-IST`

**Decision.** `fixtures/reference` holds a certificate, CMS and reply frame
produced by the earlier Python signer's own code for a throwaway identity, and
tests assert that Swakshar reproduces the CMS and the reply exactly. Before
signing, a token certificate must re-encode to its original bytes, or
Swakshar refuses it with a clear message.

**Reasoning.** The Python signer is what the GST portal accepts today, and
RSA PKCS#1 v1.5 is deterministic, so byte equality with its output is the
strongest evidence available without a live portal. The builder embeds the
certificate by re-encoding the parsed structure; a certificate that does not
round-trip would reach the portal with bytes its CA never signed and fail
verification there, so it is refused locally instead. The fixture keys
existed only in memory, so the fixtures can sign nothing else.

## Frontend lint follows React's automatic JSX runtime

`2026-09-26-00-44-00-IST`

**Decision.** `.oxlintrc.json` turns off `react/react-in-jsx-scope`.

**Reasoning.** The rule exists for the classic JSX transform, where every file
had to import `React`. The app compiles with `"jsx": "react-jsx"`, the
automatic runtime, so the rule reports on correct code. Disabling it in the
shared config is choosing the ruleset that matches the runtime, the same thing
ESLint's `jsx-runtime` preset does; it is not a per-line suppression.

## Cargo.lock is resolved with a seven-day publish-age gate

`2026-09-26-00-43-00-IST`

**Decision.** Until Rust 1.100 (2026-11-12) ships `min-publish-age`, the lock
file is generated with nightly Cargo:
`CARGO_REGISTRY_GLOBAL_MIN_PUBLISH_AGE="7 days" cargo +nightly -Zmin-publish-age generate-lockfile`.
Stable Cargo then builds against that lock with `--locked`.

**Reasoning.** Exact pins cover direct dependencies only; transitive ones would
otherwise resolve to whatever was published minutes ago (a `cc` release was
17 minutes old during the first resolution). The nightly flag applies the
owner's one-week rule to every package in the graph with no extra tooling.

## Toolchains pinned to Rust 1.98.0, pnpm 11.23.0, Node 24 LTS

`2026-09-26-00-42-00-IST`

**Decision.** `rust-toolchain.toml` pins 1.98.0, `packageManager` pins pnpm
11.23.0, `.nvmrc` pins Node 24.21.0 for CI.

**Reasoning.** These match what the owner's Mac already has installed, so
nothing new is downloaded onto a nearly full disk, and all are more than seven
days old. The research had suggested Rust 1.98.1 and pnpm 12.4.2; neither
adds anything this project needs yet. Bumps go through a new entry.

## The in-app updater waits for the first public release

`2026-09-26-00-41-00-IST`

**Decision.** `tauri-plugin-updater` is not wired in yet.

**Reasoning.** Update manifests on a private repository are invisible to
users, and updater artifacts need a signing key that does not exist yet. The
updater, its key and `latest.json` arrive together with the first public
release, as the go-public checklist in `docs/RELEASING.md` says.

## One request at a time, signtype 1 only, five minutes to decide

`2026-09-26-00-40-00-IST`

**Decision.** The app answers one request at a time, refuses any `signtype`
other than `1`, and cancels a request nobody answers within five minutes.

**Reasoning.** Every observed portal flow sends `signtype=1`; the other path
was only ever known from decompiled code, which the clean-room rule keeps out.
A single pending request keeps the approval window unambiguous about what is
being signed, and the timeout frees the portal page instead of hanging it.

## The PIN is asked for every signature and never kept

`2026-09-26-00-39-00-IST`

**Decision.** The token session is opened, logged in, used and logged out for
each signature. The PIN becomes a `SecretString` on arrival and is never
stored, logged or sent back to the webview.

**Reasoning.** A session left open lets anything that reaches the signer sign
without the user. Asking each time costs a few keystrokes on rare filings and
removes that class of attack entirely.

## The local CA key is never written to disk

`2026-09-26-00-38-00-IST`

**Decision.** Each install mints an ECDSA CA constrained to `127.0.0.1` and
`localhost` with path length 0, signs one 800-day leaf, and drops the CA key.

**Reasoning.** A trusted CA key on disk would let anyone who copies it
impersonate any website to this user. With no key and name constraints, the
worst case is impersonating the signer on loopback, which already needs local
access. Renewal every two years costs one password prompt.

## Owner choices for the build

`2026-09-25-15-45-00-IST`

**Decision.** React for the UI, `MIT OR Apache-2.0` licence, macOS first, and
the whole app written by the agent with macro commits while the owner does
all verification and release.

**Reasoning.** Stated by the owner on 2026-09-25 after the research. React is
the owner's strongest frontend stack and Tauri renders it in the system
webview. The dual licence is the Rust norm, carries Apache's patent grant, and
qualifies for SignPath's free signing. macOS comes first because the vendor
signer there is dead, while the Windows one still works.

## Release builds run on one native GitHub-hosted runner per OS

`2026-09-25-15-05-00-IST`

**Decision.** A `vX.Y.Z` tag runs `.github/workflows/release.yml`: a draft
release, then parallel tauri-action builds on `macos-15` (universal, signed
and notarized), `windows-2025` (x86_64, NSIS) and `ubuntu-22.04` (x86_64, deb,
rpm, AppImage), then `SHA256SUMS` and, once public, build provenance. Manual
dispatch builds the same matrix as workflow artifacts. The owner installs the
draft's assets and publishes it by hand.

**Reasoning.** The Rust code is portable, but a desktop release is platform
bundles plus platform signing: notarization runs only on macOS, NSIS and
Authenticode belong on Windows, and Linux packages must link a known WebKitGTK
and glibc baseline. Tauri cannot build macOS bundles on other systems and its
Windows cross-build is experimental. The app also has OS-specific code (trust
store, driver discovery, entitlements) that only real runs exercise. macOS
ships universal so users with x86_64-only token drivers can fall back to
Rosetta. Windows ships x64 only because Windows on Arm emulates x64 and the
vendor DLLs are x64. Linux starts at x64 on 22.04 for a glibc 2.35 baseline
and adds arm64 once public, because arm runners are public-only. While private
on the Free plan, macOS minutes count 10x and environment secrets, rulesets
and attestations are unavailable, so CI runs Linux on every change and the
other systems on dispatch, signing secrets start as repository secrets, and
attestation switches on by itself with public visibility.

## macOS builds are signed and notarized from the first build

`2026-09-25-15-04-00-IST`

**Decision.** Every macOS artifact, preview builds included, is signed with a
Developer ID Application certificate and notarized through an App Store
Connect API key. The app runs with the hardened runtime plus
`com.apple.security.cs.disable-library-validation`, and gains no other
entitlement until one is proven necessary.

**Reasoning.** The owner holds an Apple Developer membership, so there is no
reason to ship unsigned builds even privately. Unsigned builds teach users to
click through Gatekeeper, the exact habit an impostor build needs, and this
app can make legally binding signatures. Notarization requires the hardened
runtime, whose library validation refuses token drivers signed by other teams
(the HYP2003 driver is signed by team `S47T4UESP3`) or not signed at all;
disabling library validation is the narrow fix. The API key keeps the owner's
Apple ID password out of CI.

## The product is named Swakshar

`2026-09-25-15-03-00-IST`

**Decision.** Product name Swakshar. GitHub org `swakshar-app`, repository
`swakshar-app/swakshar`, crates `swakshar-*` with `publish = false`, CLI binary
`swakshar`, Tauri identifier `com.swakshar.desktop`, domains `swakshar.com` and
`swakshar.in`.

**Reasoning.** Swakshar means signature in Bengali, Marathi, Odia and
Assamese, and its Sanskrit roots read "one's own mark": the user's own DSC, on
the user's own machine, signing only what the user approves. It does not
contain GST, so the planned MCA, Income Tax and PDF signing fit under it. On
2026-09-25 it was free on crates.io, npm and Homebrew, all of .com, .in, .app,
.dev and .org were unregistered, and no software product used it. The bare
GitHub handle belongs to a dormant account, hence the `swakshar-app` org. The
identifier avoids a trailing `.app`, which Tauri warns about because it clashes
with the macOS bundle extension, and it is fixed now because changing it later
breaks updates. Mohar was the runner-up; its handles and domains were taken. A
trademark search in classes 9 and 42 is due before the repository goes public.

## Private repository first, public at completion

`2026-09-25-15-02-00-IST`

**Decision.** Work happens in a private repository that becomes public when
the work is complete. From the first commit the history stays publishable: no
real identities, no secrets, no vendor material.

**Reasoning.** The owner wants to finish before exposing the work. Making a
repository public exposes every commit ever made, so anything that must never
be public has to stay out from the start; scrubbing it later means rewriting
history. Plan-gated GitHub features (environment secrets, required reviewers,
rulesets, attestations, arm runners, SignPath signing) wait for the public
phase, and the workflows are written so that the switch needs configuration,
not rewrites.

## Tauri 2 is the desktop shell

`2026-09-25-15-01-00-IST`

**Decision.** The GUI is a Tauri 2 app (2.11 line) with a TypeScript frontend.
Tauri 3 gets evaluated after it leaves alpha.

**Reasoning.** Chosen by the owner after the research comparison with Slint,
egui, iced and Dioxus. The webview gives full design control with the
TypeScript and React skills the owner already has, and it fits the
TypeScript-only, pnpm and oxlint standards. Every desktop integration the app
needs (tray, single instance, autostart, notifications, signed updates,
installers, notarization) is official, and the work compounds with the owner's
Tauri learning. The accepted costs are three webview engines to test and
JavaScript inside the trust boundary, handled with per-window capabilities, a
strict CSP, no remote content and validation of every command in Rust.

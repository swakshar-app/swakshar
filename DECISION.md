# Decisions

Newest first. Each entry has a title, an IST timestamp, the decision and the
reasoning. Supersede an entry with a new one; never rewrite an old entry. Keep
this file under 300 lines by moving older entries into `decisions/` with an
index.

## The app icon is a token that ends in a pen nib, drawn from SVG

`2026-09-26-20-37-00-IST`

**Decision.** The mark is a DSC token whose end is a fountain-pen nib,
drawing a gold signature on the brand teal square. The masters are
`app/src-tauri/icons/source/app-icon.svg` and `tray-template.svg`, and
`pnpm run icons` builds every bundle icon and the menu bar template with
Tauri's own `tauri icon`. The hand-rolled rasterizer and PNG, ICNS and ICO
encoders in `scripts/` are removed.

**Reasoning.** The first mark, a seal ring around a squiggle, read as a
wave. The new one says what the app does, your token signs, and keeps its
silhouette at 32 pixels, where the signature becomes an underline. Tauri's
generator is the official tool for these formats, so the repository no longer
maintains its own image encoders.

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

## Older decisions

Archived in `decisions/`, newest first:

- [2026-09-25](decisions/2026-09-25.md): owner choices for the build, release
  runners, macOS signing and notarization, the name Swakshar, private
  repository first, Tauri 2 as the shell.

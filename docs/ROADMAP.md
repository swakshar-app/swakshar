# Roadmap

What ships next, in order. Settled choices live in `DECISION.md`; this file
only says what is planned and what it waits on.

## v0.1.0: first signed macOS release

Built and on `develop`:

- Portal protocol, `signtype=1` CMS pinned byte for byte to the reference
  signer, PKCS#11 token access on one thread, per-install local CA, loopback
  `wss://` server with Origin and Host checks.
- Menu bar app: approval window, guided setup, tokens recognised on USB with
  driver guidance, activity, settings, help checklist, `swakshar` CLI.
- CI: Rust, frontend and cargo-deny gates; signed and notarized release
  workflow for macOS, Windows and Linux.
- Token drivers: users install their token maker's driver once, as they do
  today for other signers. Swakshar recognises the token on USB, names the
  driver it needs, and picks it up as soon as it is installed.
- In-app updates, local notifications and a diagnostic report, as
  `docs/DISTRIBUTION.md` describes (being built).

Waiting on the owner:

1. The updater key's two GitHub secrets (its public key is in the app, and
   Apple signing is done).
2. The manual acceptance run on a real token and the real portal
   (`docs/TESTING.md`).
3. Make the repository public, tag `v0.1.0`, install the draft assets on a
   clean Mac, and publish without announcing.
4. Publish `v0.1.1` and confirm 0.1.0 offers, downloads and restarts into
   it (`docs/RELEASING.md`).
5. Announce.

## v0.2.0: no driver install for ePass2003 and HYP2003 tokens

A native Rust driver for the Feitian ePass2003 family, which includes the
Hypersecu HYP2003, so the most common Indian tokens sign with nothing to
install: the USB transport in Rust (`nusb` is already in the app), the
token's secure channel and commands in Rust, behind the existing token
interface so PKCS#11 drivers keep working for every other brand.

- Known so far (2026-09-26): the HYP2003 (USB `0x2CCF:0x080A`) is an
  ePass2003-family token. OpenSC's open-source ePass2003 driver reaches the
  chip but cannot read tokens issued by Indian CAs, which use the vendor's
  own storage layout. macOS and the upstream CCID driver do not list the
  `0x2CCF` vendor ID.
- Gate: the storage layout must come from a source this repository may use,
  as the clean-room rule in `AGENTS.md` requires: documentation or written
  permission from Feitian or Hypersecu, or a published specification. No
  decompiled vendor code. First step: write to both vendors.
- Done when: the HYP2003 and an ePass2003 sign on a Mac with no vendor
  software installed, and the signature is byte-identical to the vendor
  driver's for the same content and time.

## v0.3.0 (idea): a signing test page on swakshar.app

An idea, not yet committed to: a public page, `https://swakshar.app/test`, in the spirit of eMudhra's
emBridge test tool: it checks a computer end to end without the GST portal.
It finds the local signer on the portal's ports, checks the greeting, sends a
test request, verifies the returned signature in the browser and shows the
certificate that signed.

- The page is static and sends nothing anywhere; the signature is checked in
  the browser and discarded.
- Swakshar answers `https://swakshar.app` for test requests only: content
  that starts with `swakshar-test:` and a random value, never a PAN or a
  document fingerprint. The approval window labels them as a test.
- The allowed origin, the test content rule and the approval label get a
  `DECISION.md` entry and a `THREAT_MODEL.md` row before this ships.

## Mac App Store (feasibility first)

Publish on the Mac App Store alongside the notarized download, after a
sandbox feasibility spike. The steps and open questions are in
`docs/DISTRIBUTION.md`.

## Later

- Downloading and running a token maker's driver inside Swakshar, for brands
  without a native driver, once a maker offers a stable, signed download
  that can be pinned by hash. Today Get the driver opens the maker's
  official page (0.1.8), and only Hypersecu's page is public.
- Windows and Linux releases, Windows signing through SignPath and an
  `ubuntu-22.04-arm` build (`docs/RELEASING.md`, Going public).

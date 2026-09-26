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

Waiting on the owner:

1. Apple signing secrets in GitHub (`docs/RELEASING.md`), then one manual
   release run to confirm signing and notarization.
2. The manual acceptance run on a real token and the real portal
   (`docs/TESTING.md`).
3. Tag `v0.1.0`, install the draft assets on a clean Mac, publish.

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

## Later

- One-click install of a token maker's driver from its official download,
  for brands without a native driver.
- Windows and Linux releases, Windows signing through SignPath, an
  `ubuntu-22.04-arm` build, and the in-app updater (`docs/RELEASING.md`,
  Going public).

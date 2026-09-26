# Swakshar

Sign GST portal filings with your own DSC token, on your own computer, only
after you approve each request. Open source. macOS first.

> Unofficial software. Not affiliated with, endorsed by, or supported by GSTN,
> the GST Council, Infosys, eMudhra, or any Certifying Authority. It signs only
> with your own DSC, only after you approve each request. You are responsible
> for what you sign. No warranty.

Swakshar (स्वाक्षर) means "signature" in Bengali, Marathi, Odia and Assamese;
its Sanskrit roots read "one's own mark".

## What it does

When you register a DSC or file a return with DSC, the GST portal asks a local
signer program on your computer (`wss://127.0.0.1:1585`) for a signature.
Swakshar is that signer:

- It shows which site is asking, what will be signed, and which certificate
  matches the PAN, then waits for you to press Sign.
- It signs on your USB token through the token's own driver. The private key
  never leaves the token and the PIN is never stored.
- It answers only GST portal pages, only on `127.0.0.1`. The one request it
  makes to the internet is the update check, which you can turn off. No
  telemetry.

## Status

Pre-release. The desktop app and the command-line tool build on macOS; the
first signed release is being prepared. Windows and Linux follow. See
[docs/ROADMAP.md](docs/ROADMAP.md) for what comes next.

## First run

1. **Connect your DSC token.** Swakshar recognises the token on USB, even
   before its driver is installed, and says which driver it needs. Drivers
   for ePass2003, HYP2003, SafeNet eToken, Watchdata ProxKey, TrustKey and
   mToken are found automatically once installed; others can be added in
   Settings. Drivers come from your token's supplier, not from Swakshar.
2. **Check your certificate.** Listed without a PIN.
3. **Install the local certificate.** macOS asks for your password once. The
   certificate is valid only for `127.0.0.1` and `localhost`; its CA key is
   never saved, so it can never vouch for a real website.
4. **Turn on signing.** New installs start with signing off, so nothing
   listens on your computer until you choose. Swakshar remembers your choice.
5. **Allow Swakshar on the portal.** Chrome, Edge and Brave ask once whether
   the GST site may connect to apps on this device. Choose Allow.

Swakshar lives in the menu bar and keeps running when its window is closed.
To update it, choose **Quit Swakshar** from its menu bar icon first, then
drag the new version into Applications; macOS cannot replace an app that is
still running. In-app updates replace this step once releases are public.

## Browsers

| Browser | Extra step |
|---|---|
| Chrome, Edge, Brave | Allow the local network (apps on this device) prompt on the GST site |
| Firefox | None; it trusts certificates installed in macOS |
| Offices | Chrome policy `LoopbackNetworkAllowedForUrls` with `https://[*.]gst.gov.in` |

## Command line

The `swakshar` binary serves power users and diagnostics:

```sh
swakshar doctor              # drivers, tokens, certificates, local certificate
swakshar setup --trust       # create and trust the local certificate
swakshar serve               # portal signer with approval in the terminal
swakshar selftest            # sign a test text on the token and verify it
swakshar probe               # behave like the portal against a running signer
```

## Build from source

Requirements: Rust via rustup (the toolchain in `rust-toolchain.toml`),
Node.js 24 (`.nvmrc`), pnpm 11.23.0, Xcode Command Line Tools.

```sh
pnpm install --frozen-lockfile
pnpm run check                               # typecheck, lint, build UI, line limit
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm tauri build                             # app bundle in target/release/bundle
cargo build -p swakshar-cli --release        # target/release/swakshar
```

## Documentation

| Document | Covers |
|---|---|
| [docs/PROTOCOL.md](docs/PROTOCOL.md) | The portal's local signer protocol, from public sources |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crates, threads, the approval lifecycle, files on disk |
| [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) | Assets, adversaries, controls, residual risk |
| [docs/TESTING.md](docs/TESTING.md) | Automated tests and the manual acceptance checklist |
| [docs/RELEASING.md](docs/RELEASING.md) | Signing, notarization and the release workflow |
| [DECISION.md](DECISION.md) | Every settled decision and why |
| [SECURITY.md](SECURITY.md) | Reporting vulnerabilities |

## Licence

[Apache License 2.0](LICENSE).

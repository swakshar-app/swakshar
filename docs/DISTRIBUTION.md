# Distribution, updates and notifications

How Swakshar reaches people and stays current. This is a plan: items move
into `DECISION.md` as the owner settles them. Ordering lives in
`docs/ROADMAP.md`.

## Channels

| Channel | Build | Updates | Status |
|---|---|---|---|
| Direct download from swakshar.app and GitHub Releases | Developer ID signed, notarized `.dmg` (universal) | In-app updater | Working: every release run produces it |
| Mac App Store | Sandboxed build, Mac App Distribution signed `.pkg` | The App Store | Feasibility first (below) |
| Windows, Linux | `.msi`/`.exe`, `.deb`/`.rpm`/`.AppImage` | In-app updater | Built unsigned today; signing after going public |

The direct download stays the main channel: it supports every token whose
maker ships a PKCS#11 driver. The store build can only reach tokens through
macOS itself.

## In-app updates (direct download)

Tauri's official `tauri-plugin-updater`, driven from Rust like the driver
picker, so no window gains updater permissions.

1. **Check.** At launch and every 24 hours, a `GET` of
   `https://swakshar.app/updates/latest.json`. The request carries only the
   platform, architecture and current version in the URL; no install ID, no
   token or certificate data. Settings gets "Check for updates
   automatically" (on by default) and Help gets "Check now". This stays the
   only outbound request Swakshar makes, as `AGENTS.md` requires.
2. **Tell.** Home shows a banner and the menu bar gains "Update to X.Y.Z".
   Nothing downloads without the user's say-so.
3. **Install.** On "Install and restart": download, verify the update's
   signature against the public key built into the app, replace the app,
   relaunch. Refused while a signature request is waiting. "Install when I
   quit" is the other choice.
4. **Trust.** Two signatures guard every update: Tauri's (ed25519 key held
   only in GitHub secrets) and Apple's (Developer ID plus notarization on the
   `.app` inside). A tampered or unsigned update is rejected before it is
   written.

What it needs:

- An updater key pair: `pnpm tauri signer generate`. The private key and its
  password go into GitHub secrets (`TAURI_SIGNING_PRIVATE_KEY`,
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, names the release workflow already
  reads) and the owner's password manager; the public key goes into
  `tauri.conf.json`. **Losing the private key strands every install**: they
  can only update by downloading again by hand.
- `bundle.createUpdaterArtifacts: true`, so macOS releases also produce
  `Swakshar.app.tar.gz` and its `.sig`, and the release workflow writes
  `latest.json`.
- Hosting on swakshar.app: `latest.json` and the update archives on a
  static host (for example Cloudflare Pages or R2). The URL then never
  depends on where the code lives or whether the repository is public.
- A publish step: the release workflow uploads the archives and
  `latest.json` only after the draft release is published, so nobody is
  offered an update that is still being tested.

**Ship it in v0.1.0.** A copy without the updater can only be updated by
downloading again, so every early adopter would have to reinstall once.
Hosting on swakshar.app removes the reason for waiting until the repository
is public (the 2026-09-26 updater entry in `DECISION.md`).

## Notifications

Tauri's official `tauri-plugin-notification` posts local macOS
notifications; no server is involved. macOS asks the user's permission the
first time.

| When | Notification |
|---|---|
| An update is ready | "Swakshar X.Y.Z is ready. Install and restart?" |
| A signing certificate expires within 30 days (checked daily) | "Your DSC for NAME expires on DATE. Renew it with your CA." |
| A signature request arrives while Swakshar's windows are hidden behind full-screen apps | "The GST portal wants your signature." |

**No remote push.** Push through Apple's service would need a server that
knows every install and its device token, which is exactly the tracking
Swakshar promises not to do, and there is nothing server-side to announce
that the update check does not already carry. Tauri's official notification
plugin is used for local notifications only.

## Mac App Store

1. **Feasibility spike** (engineering, with a real token). Build a sandboxed
   variant and prove: signing through CryptoTokenKit
   (`com.apple.security.smartcard`) with the token maker's CryptoTokenKit
   extension, byte-identical to the PKCS#11 path; the loopback signer under
   `com.apple.security.network.server`; installing the loopback CA from the
   sandbox, or a guided Keychain Access fallback. The updater is compiled
   out of this build: store apps may not update themselves.
2. **Apple setup** (owner). App ID `app.swakshar.desktop`; Mac App
   Distribution and Mac Installer Distribution certificates; a Mac App Store
   distribution profile.
3. **App Store Connect record** (owner). New macOS app "Swakshar", SKU
   `swakshar-macos`; category Finance; free; App Privacy "Data Not
   Collected"; privacy policy and support URLs on swakshar.app; EU trader
   status, or no EU availability.
4. **Build and upload** (CI). Sandboxed build with the profile embedded,
   signed `.pkg`, uploaded with the App Store Connect API key.
5. **TestFlight** on the owner's Mac with a real token.
6. **Submission.** Screenshots, description, keywords; review notes with a
   screen recording, since reviewers have no DSC token; the encryption
   questions (TLS and digital signatures only).
7. **Review and release.**

## Decisions the owner needs to make

1. Host updates on swakshar.app, and ship the updater in v0.1.0.
2. Update checks on by default, with the Settings switch to turn them off.
3. Start the App Store spike after v0.1.0 ships.

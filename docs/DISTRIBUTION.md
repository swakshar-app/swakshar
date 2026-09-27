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
picker, so no window gains updater permissions. Ships in v0.1.0.

1. **Check.** Ten seconds after launch, then every hour, and whenever the
   main window opens if the last check is over fifteen minutes old. One
   `GET` of `latest.json` from the latest published GitHub release; the URL
   carries only platform, architecture and current version, no install ID,
   token or certificate data. Settings has "Check for updates
   automatically" (on by default) and Help has "Check now". This stays the
   only outbound request Swakshar makes, as `AGENTS.md` requires.
2. **Tell.** A card at the bottom right of the main window, on every page,
   shows "Swakshar X.Y.Z is available" with See changes (the release notes
   carried in `latest.json`) and Download; the menu bar gains "Update to
   X.Y.Z". Nothing downloads without the user's click. The card can be
   dismissed until the update reaches its next step.
3. **Install.** Once downloaded: Restart now, or Restart when idle, which
   waits until no request is waiting and no GST page has connected for five
   minutes. Never while a request is waiting. Signing comes back as the user
   left it.
4. **Trust.** Two signatures guard every update: Tauri's (ed25519 key held
   only in GitHub secrets and the owner's password manager) and Apple's
   (Developer ID plus notarization on the `.app` inside). A tampered or
   unsigned update is rejected before it is written.

What it needs:

- An updater key pair (owner): `pnpm tauri signer generate`. The private key
  and its password go into GitHub secrets (`TAURI_SIGNING_PRIVATE_KEY`,
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, names the release workflow already
  reads) and the password manager; the public key goes into the app.
  **Losing the private key strands every install**: they could only update
  by downloading again by hand.
- `bundle.createUpdaterArtifacts: true`, so releases also produce the update
  archives and `.sig` files, and the release workflow writes `latest.json`
  into the draft release, linking each archive by its release download
  address rather than the rate-limited GitHub API.
- Hosting: GitHub Releases. The app asks
  `https://github.com/swakshar-app/swakshar/releases/latest/download/latest.json`,
  which only resolves once the repository is public and always points at
  the newest **published** release, so a draft under test is never offered.
  A debug build can point at any HTTPS test manifest with
  `SWAKSHAR_UPDATE_ENDPOINT` to try the flow while the repository is
  private; the updater refuses plain HTTP.
- An update test before announcing: publish `v0.1.0`, install it, publish
  `v0.1.1`, and watch 0.1.0 update itself. Release candidates cannot stand
  in for this: `releases/latest` skips pre-releases and the Windows
  installer format rejects most pre-release versions.

## Notifications

Tauri's official `tauri-plugin-notification` posts local macOS
notifications; no server is involved. macOS asks the user's permission the
first time, and Settings has "Show notifications" (on by default).

| When | Notification |
|---|---|
| An update is ready | "Swakshar X.Y.Z is ready. Restart to update." |
| A signing certificate expires within 30 days (checked daily) | "Your DSC for NAME expires on DATE. Renew it with your CA." |
| A signature request arrives while Swakshar's windows are hidden behind full-screen apps | "The GST portal wants your signature." |

**No remote push and no telemetry.** Push through Apple's service would need
a server that knows every install and its device token, which is the
tracking Swakshar promises not to do.

## Diagnostics for bug reports

Help gets "Copy diagnostic report" and "Report a problem", so people, and
forks, can debug a machine without Swakshar ever sending anything itself.
The report holds the app version, macOS version and architecture, the
checklist, the driver list with load errors, and recent log lines; never
names, PANs, token serials or the activity history. Report a problem opens
a new GitHub issue in the browser; the user pastes and reviews the report
before submitting.

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

## Settled

The owner's answers of 2026-09-27 are recorded in `DECISION.md`: the
updater ships in v0.1.0, served from GitHub Releases, with checks on by
default; v0.1.0 ships only once the whole release process works, updates
included; notifications are local and can be turned off; diagnostics are
copied by the user, never sent; the App Store is low priority, after
v0.1.0.

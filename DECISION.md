# Decisions

Newest first. Each entry has a title, an IST timestamp, the decision and the
reasoning. Supersede an entry with a new one; never rewrite an old entry. Keep
this file under 300 lines by moving older entries into `decisions/` with an
index.

## Releases stay on 0.1.x until the owner calls for 0.2.0

`2026-09-27-17-28-00-IST`

**Decision.** v0.1.0 is the first release. Every release after it bumps
the patch number only (0.1.1, 0.1.2, and so on) until the owner asks for
0.2.0. Roadmap items labelled v0.2.0 or later ship in 0.1.x patches when
they are ready, unless the owner decides otherwise.

**Reasoning.** Asked for by the owner on 2026-09-27 at the first release.
One stable line keeps the updater path simple and signals to users that
Swakshar is still settling in.

## Dependencies stay current within seven days, with two holds

`2026-09-27-17-14-00-IST`

**Decision.** Every dependency moves to its newest release that is at least
seven days old, through Dependabot's cooldown and the lockfile gate. Two
holds: `der`, `x509-cert`, `spki` and `const-oid` stay on the lines
`cms` 0.2.3 needs until a stable `cms` 0.3 exists, and `@types/node` stays
on the Node major in `.nvmrc`. Dependabot ignores those updates.

**Reasoning.** The owner wants dependencies current. `cms` 0.2.3 is the
newest stable `cms` and depends on `der` 0.7, so the newer formats cannot
be mixed in, which is why the grouped update failed to compile. Types for a
newer Node than the one CI runs would let code use APIs that do not exist
at run time. The byte-for-byte parity tests catch any change in what gets
signed when these do move.

## The first update test is 0.1.0 to 0.1.1, before announcing

`2026-09-27-11-24-00-IST`

**Decision.** The end-to-end update test publishes `v0.1.0`, installs it
from the release, publishes `v0.1.1`, and confirms 0.1.0 updates itself,
all right after the repository goes public and before the release is
announced. This replaces the release-candidate test in the entry below.

**Reasoning.** `releases/latest` serves only the newest published,
non-prerelease release of a public repository, so release candidates are
never offered, and the Windows installer format rejects most pre-release
version numbers. Two real versions published quietly test exactly what
users will run.

## Updates ship in v0.1.0 from GitHub Releases; nothing else leaves the Mac

`2026-09-27-02-15-00-IST`

**Decision.** v0.1.0 includes `tauri-plugin-updater`, checking the latest
published GitHub release's `latest.json` ten seconds after launch, hourly,
and when the main window opens, on by default with a Settings switch.
Updates download on the user's click and install on Restart now or Restart
when idle, never during a request. Local notifications
(`tauri-plugin-notification`) are on by default with a Settings switch. Bug
reports use a diagnostic report the user copies; Swakshar sends no
telemetry and uses no remote push. v0.1.0 ships only when the release
process works end to end, including a release-candidate update. The App
Store follows later. This supersedes "The in-app updater waits for the
first public release"; `docs/DISTRIBUTION.md` holds the design.

**Reasoning.** Owner's answers on 2026-09-27. A copy released without the
updater can only be updated by downloading it again, so the updater has to
be in the first release. GitHub Releases needs no other host, and
`releases/latest` only ever points at a published release. A diagnostic
report the user reviews helps forks and bug reports without breaking the
promise that nothing leaves the Mac except the update check.

## The bundle identifier is app.swakshar.desktop

`2026-09-27-01-42-00-IST`

**Decision.** The Tauri and macOS bundle identifier, and with it the per-user
data and log folders, is `app.swakshar.desktop`. The project's domain is
`swakshar.app`. This supersedes the identifier and domains in "The product
is named Swakshar" (`decisions/2026-09-25.md`); the rest of that entry
stands.

**Reasoning.** The owner registered `swakshar.app` on 2026-09-26, and a
reverse-DNS identifier should come from a domain the project holds. It
changes before the first release and before an App Store Connect record
fixes it for good. Pre-release installs start fresh in the new folder: set
up once more, and remove the old local certificate first.

## Signing is off until the user turns it on

`2026-09-27-01-29-00-IST`

**Decision.** A fresh install starts with signing off: nothing listens on
the portal's ports. Turning signing on or off, from Home or the menu bar,
is saved, and each launch starts the way the user left it. Quit closes the
listener before anything else.

**Reasoning.** Asked for by the owner on 2026-09-26. A listener on
loopback is small attack surface, and every signature still needs the user's
approval and PIN, but there is no reason to listen before the user has set
Swakshar up and chosen to sign. Remembering the choice keeps people who sign
daily from turning it on after every restart.

## Older decisions

Archived in `decisions/`, newest first:

- [2026-09-26](decisions/2026-09-26.md): OpenSC not bundled, the token-nib
  icon, token drivers from their makers, cargo-deny scope, Apache-2.0 only,
  byte-for-byte parity with the Python signer, lint and lockfile policy,
  toolchain pins, the updater's first plan, one request at a time, the PIN
  never kept, the local CA key never written.
- [2026-09-25](decisions/2026-09-25.md): owner choices for the build, release
  runners, macOS signing and notarization, the name Swakshar, private
  repository first, Tauri 2 as the shell.

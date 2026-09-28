# Decisions

Newest first. Each entry has a title, an IST timestamp, the decision and the
reasoning. Supersede an entry with a new one; never rewrite an old entry. Keep
this file under 300 lines by moving older entries into `decisions/` with an
index.

## The token driver is closed before the process exits

`2026-09-28-13-30-00-IST`

**Decision.** Every exit shuts the token thread down after the signer
stops: the thread finishes the driver call it is in, drops queued jobs,
calls `C_Finalize` on every module and signals when done. The exit waits up
to two seconds for it; the menu bar Quit's forced exit moves from three to
five seconds so it never cuts that wait short. The command line does the
same on Ctrl-C.

**Reasoning.** Two crash reports from 0.1.3 on 2026-09-28 showed the same
thing: Command-Q ran `exit`, which ran the vendor driver's teardown on the
main thread while the token thread was inside `C_GetSlotList`, and the
token thread crashed with SIGSEGV. The fix only changes our own shutdown
order; nothing inside the driver was examined beyond the crash reports'
stack traces.

## Every exit runs one orderly shutdown

`2026-09-28-00-30-00-IST`

**Decision.** The server stops through a `Stopper`: the port closes at
once, a waiting request is answered "signing canceled", idle pages get a
close frame, and connections left after one second are cut off. The app
runs `quit::on_exit` on Tauri's exit event, so Quit in the menu bar,
Command-Q, Quit in the Dock, logging out and restarts into an update all
answer the waiting page and wait up to two seconds for open pages to
receive their reply.

**Reasoning.** The owner asked on 2026-09-27 for Command-Q and Quit in the
Dock to be as clean as the menu bar's Quit. macOS ends those through
`applicationWillTerminate`, which Tauri reports only as its exit event, so
the shutdown has to live there. Working on it showed the menu bar's Quit
was not clean either: stopping aborted every connection, so the cancel
reply could be lost before it was written.

## Updater links point at release downloads

`2026-09-27-18-40-00-IST`

**Decision.** The release workflow rewrites `latest.json` after the builds
so each platform links to `github.com/<repo>/releases/download/<tag>/<file>`
instead of the `api.github.com` asset links tauri-action writes for drafts.
It fails the release when a link names a file the release does not have.

**Reasoning.** The API links work, but anonymous GitHub API calls are
limited to 60 an hour per network address, so an office full of Macs
updating together could be refused. Release downloads have no such limit.
Signatures are untouched: they cover the files, not the links.

## Dependabot's glib alert is accepted until Tauri moves on

`2026-09-27-18-41-00-IST`

**Decision.** GHSA-wrw7-89jp-8q8g (unsound `glib::VariantStrIter`, fixed in
glib 0.20) is dismissed as a tolerable risk. Revisit when Tauri's Linux
stack moves past gtk-rs 0.18.

**Reasoning.** glib 0.18 reaches Swakshar only on Linux, through Tauri's
GTK tray and window stack; macOS and Windows builds do not contain it.
Swakshar never calls the affected iterator, and the fix needs a GTK
upgrade only Tauri can make.

## Swakshar shows in the Dock while a window is open

`2026-09-27-17-55-00-IST`

**Decision.** On macOS Swakshar is a regular app, with a Dock icon, a place
in the app switcher and its own menu bar, while its main or approval window
is open or minimized, and a menu bar only app once they are all closed.
Settings has "Keep Swakshar in the Dock when its windows are closed", off by
default, which keeps it regular. Ships in 0.1.1.

**Reasoning.** The owner found on 2026-09-27 that an open Swakshar window
had no Dock icon and no Command-Tab entry: the app set the accessory policy
at launch and kept it. Showing while open matches how people find windows
on a Mac; going back to the menu bar keeps a signer that runs all day out
of the way. Quit from the app menu or the Dock goes through macOS directly
rather than the tray's orderly quit, so a waiting page sees its connection
close instead of a cancel reply; the port still closes with the process.

## Releases are titled with the bare tag

`2026-09-27-17-40-00-IST`

**Decision.** From v0.1.1 on, a release's title is its tag alone, such as
`v0.1.1`, not `Swakshar v0.1.1`. v0.1.0 keeps the title it was published
with.

**Reasoning.** Asked for by the owner on 2026-09-27. The repository already
names the app, so the product name in every title is noise in the releases
list.

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

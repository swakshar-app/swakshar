# Changelog

What changed in each release, newest first. The release workflow publishes
each version's section as its release notes, and the in-app update card shows
it under See changes.

## 0.1.5

- Fixes the main window going blank after a signature. Home stayed on
  Loading, Activity looked empty, and Help lost its checklist and version
  until Swakshar was restarted. The signature itself was always made and
  recorded.
- Pages now say Loading until their information arrives, instead of looking
  empty.
- The log no longer fills with harmless "mechanism not supported" errors
  from the token library on every signature.

## 0.1.4

- Fixes a crash when quitting. If Swakshar was checking your token at the
  moment you quit, macOS could report that it quit unexpectedly. It now
  finishes that check and closes the token driver before it exits.

## 0.1.3

- The update card shows what changed as a tidy list instead of raw text
  broken mid-sentence.
- Quitting is orderly however you quit: from the menu bar, with Command-Q,
  from the Dock or by logging out. A GST page waiting for your signature is
  told it was cancelled instead of seeing the connection drop.

## 0.1.2

- Restart after an update opens Swakshar again. Before, pressing Restart
  installed the update but left Swakshar closed until you opened it.
  Updating from 0.1.1 to this version can still leave it closed once;
  open it from Applications.
- Updates download from the release itself rather than GitHub's API, so
  several Macs on one office network are no longer at risk of being turned
  away.

## 0.1.1

- Swakshar shows in the Dock and the app switcher, with its own menu bar,
  while one of its windows is open, and goes back to the menu bar alone
  when you close them.
- Settings can keep Swakshar in the Dock with its windows closed; clicking
  the icon opens it.

## 0.1.0

The first release: a signer for the GST portal that uses your own DSC token
and asks before every signature.

- Signs GST registrations and returns on the portal with your DSC, over a
  local connection that only GST portal pages can use.
- Shows who is asking, what is being signed and which certificate matches
  the PAN, and waits for you to press Sign. The PIN is never stored.
- Recognises your token on USB, names the driver it needs, and picks the
  driver up as soon as it is installed.
- Guided setup, signing that stays off until you turn it on, a menu bar
  switch, activity history kept only on your Mac, and a help checklist.
- Updates itself: you choose when to download and when to restart.
- Local notifications for updates, certificates close to expiry and
  requests you have not seen, each of which you can turn off.
- A diagnostic report you can copy into a bug report; Swakshar sends
  nothing itself.

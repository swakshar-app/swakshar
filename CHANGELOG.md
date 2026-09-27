# Changelog

What changed in each release, newest first. The release workflow publishes
each version's section as its release notes, and the in-app update card shows
it under See changes.

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

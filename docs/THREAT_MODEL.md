# Threat Model

A DSC signature carries legal weight. Swakshar never sees the private key (it
stays on the token), but it decides **when** the key is used and **over what**.
That decision is the asset to protect.

## Assets

| Asset | Where | Loss means |
|---|---|---|
| The signing decision | Approval window, `pending.rs` | Someone else's text signed with your DSC |
| Token PIN | Typed into the approval window, briefly in memory | Unlimited signing while the token is plugged in |
| PAN, names, serials | Certificates, requests, history | Personal data exposure |
| Local TLS trust | Login keychain | Impersonation of websites, if the CA were unconstrained |
| Releases | GitHub Releases | A malicious build reaching users |

## Adversaries

1. A malicious or compromised website open in the user's browser.
2. Another account on a shared machine.
3. Malware running as the user.
4. A supply-chain attacker: a dependency, a CI runner, a release.
5. An impostor build under a similar name.

## Threats and controls

| Threat | Control in the code |
|---|---|
| A website opens `wss://127.0.0.1:1585` and asks for a signature | `Origin` must be `https://gst.gov.in` or a subdomain, checked before the upgrade (`crates/server/src/http.rs`); browsers set `Origin` themselves |
| DNS rebinding | `Host` must be `127.0.0.1:<port>` or `localhost:<port>`; the certificate only covers those names |
| A website fingerprints the signer | No CORS headers anywhere; the status page answers top-level navigations only (Fetch Metadata) |
| A compromised GST page asks for a signature | Every signature needs the user to press Sign in the approval window, which shows the site, the purpose and the exact document fingerprint |
| Request flooding | One request at a time; frames capped at 16 KiB; 32 connections at most; handshake timeouts |
| A page keeps a connection open after signing is turned off | Connections belong to the listener's `JoinSet`: turning signing off or quitting closes the port at once, answers a waiting request "signing canceled", closes idle pages, and cuts off the rest after one second (`crates/server/src/listener.rs`, `stop.rs`) |
| The page leaves mid-request | The request is abandoned and the window closes (`PendingGuard`) |
| Wrong certificate | PAN hash matching, validity and class filters; mismatches flagged in red |
| PIN lockout | Token PIN flags read before signing; last-try warning; no automatic retries |
| PIN exposure | Never stored, logged or returned to the webview; turned into a `SecretString` on arrival; cleared from the form on submit |
| A stolen CA key | There is none on disk: the CA key is generated in memory, signs one leaf, and is dropped. The CA is name-constrained to `127.0.0.1` and `localhost` |
| A stolen leaf key | Only useful to someone who can already listen on the port; file mode 0600 |
| Driver misbehaviour | Every signature is verified against the certificate before it is sent; fallback mechanism on mismatch |
| XSS in the app's own UI | No remote content, strict CSP, per-window capabilities: the approval window can call only its four commands |
| Malicious dependency | Exact pins, committed lockfiles, a seven-day release-age gate, `cargo deny` in CI |
| Tampered release | SHA-pinned actions, signed and notarized macOS builds, `SHA256SUMS`, build provenance once public |
| Tampered or malicious update | The updater verifies each archive against the public key built into the app (private key only in GitHub secrets), over HTTPS, and the app inside is Apple-signed and notarized; nothing installs without the user's click |
| The update check leaking who uses Swakshar | The request carries platform, architecture and version only; it can be turned off in Settings |

## Data kept on disk

| File | Contents | Mode |
|---|---|---|
| `tls/ca.pem`, `tls/server.pem` | Public certificates | 0644 |
| `tls/server-key.pem` | Loopback leaf key | 0600 |
| `settings.json` | Preferences | default |
| `activity.jsonl` | Time, site, purpose, masked PAN, holder, masked serial, outcome | default |

Logs record origins and outcomes, never PINs, signatures or full PANs.

## Residual risks

- Malware running as the user can already watch the screen and keyboard. No
  desktop app fixes that.
- A compromise of a real `*.gst.gov.in` page could request a signature. The
  approval window is the defence; it depends on the user reading it.
- GSTN verifies the signature against the DSC registered to the PAN. Swakshar
  cannot help with anything after the reply.
- Only browsers enforce `Origin`. Any program running as the user can
  connect claiming a GST origin; it still needs the user to approve in the
  window and to enter the token PIN, and such a program already runs as the
  user.
- Library validation is off so token drivers signed by other teams load
  (`decisions/2026-09-25.md`). Drivers load only from known install
  locations or files the user picked; a program able to change those
  already runs as the user.
- The PIN passes through the webview as a JavaScript string on its way to
  Rust, and JavaScript strings cannot be wiped. The form clears it at once,
  and it is useless without the physical token.

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
| The page leaves mid-request | The request is abandoned and the window closes (`PendingGuard`) |
| Wrong certificate | PAN hash matching, validity and class filters; mismatches flagged in red |
| PIN lockout | Token PIN flags read before signing; last-try warning; no automatic retries |
| PIN exposure | Never stored, logged or returned to the webview; turned into a `SecretString` on arrival; cleared from the form on submit |
| A stolen CA key | There is none on disk: the CA key is generated in memory, signs one leaf, and is dropped. The CA is name-constrained to `127.0.0.1` and `localhost` |
| A stolen leaf key | Only useful to someone who can already listen on the port; file mode 0600 |
| Driver misbehaviour | Every signature is verified against the certificate before it is sent; fallback mechanism on mismatch |
| XSS in the app's own UI | No remote content, strict CSP, per-window capabilities: the approval window can call only its three commands |
| Malicious dependency | Exact pins, committed lockfiles, a seven-day release-age gate, `cargo deny` in CI |
| Tampered release | SHA-pinned actions, signed and notarized macOS builds, `SHA256SUMS`, build provenance once public |

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

# Architecture

## Crates

| Crate | Job | Depends on |
|---|---|---|
| `swakshar-protocol` | Parse requests, render greeting and replies, origin rules, PAN masking, dates. Pure | none |
| `swakshar-cms` | Build and inspect the `signtype=1` CMS; base64url; `DigestInfo`; RSA SHA-1 verification | RustCrypto `cms`/`der`, ring |
| `swakshar-token` | PKCS#11 through cryptoki: driver discovery, tokens on USB and what each still needs, certificates, PAN matching, the signing thread | cms, protocol, nusb |
| `swakshar-tls` | Per-install CA and loopback leaf; macOS trust store | rcgen |
| `swakshar-server` | Loopback `wss://` server: TLS, Origin and Host checks, greeting, one request at a time | protocol, rustls, tungstenite |
| `swakshar-cli` | `swakshar` binary: doctor, setup, serve, selftest, probe | all of the above |
| `swakshar-app` | Tauri shell: tray, windows, commands, approval lifecycle | all of the above |

The React UI lives in `app/src`, one bundle for both windows.

## Threads and tasks

```text
browser --wss--> server task (tokio, one per connection)
                   | Host and Origin checked, greeting sent, request parsed
                   v
                 Broker::handle
                   app: pending::run   cli: terminal prompt
                   | inventory from the token thread, candidates ranked by PAN
                   | "sign-request" event, approval window shown
                   v
                 approve_request command (user pressed Sign)
                   | SignJob to the token thread
                   v
                 token thread (one std thread owns every PKCS#11 module)
                   | login, build CMS, sign, verify, logout
                   v
                 pending::finish sends the reply frame to the waiting task
```

PKCS#11 drivers are often not thread-safe, so exactly one thread ever calls
them (`TokenService`). Everything else talks to it through a channel.

## The approval lifecycle

1. `pending::run` takes the single approval slot atomically before any await,
   so a second request is refused while one waits, as is any `signtype` other
   than `1`.
2. It reads tokens (no PIN), ranks certificates with `candidates`, stores a
   `Pending`, emits `sign-request`, and shows the approval window.
3. It waits up to five minutes for the reply channel.
4. `refresh_request` re-reads the tokens when one is plugged in after the
   request arrived. `approve_request` signs. A wrong PIN keeps the request
   open and reports the token's PIN flags. Success calls `finish` with the
   reply.
5. `cancel_request`, closing the window, the timeout, or the page leaving
   (`PendingGuard` on drop) all call `finish` too. `finish` is idempotent:
   the first caller wins.
6. `finish` records history, emits `sign-finished`, and hides the window.

## Signing on and off, and quitting

Signing starts off on a fresh install. Turning it on or off, from Home or
the menu bar, starts or stops the listener and saves `signingEnabled`, so the
next launch starts the same way. Stopping (`swakshar_server::Stopper`) closes
the port at once; a request still waiting is answered "signing canceled",
idle pages get a close frame, and whatever is left after one second is cut
off. Every exit, whether Quit in the menu bar, Command-Q, Quit in the Dock,
logging out or a restart into an update, reaches `quit::on_exit` on
Tauri's exit event: it answers a waiting page, stops the listener and waits
up to two seconds for open pages to receive their last reply. Quit from the
menu bar hides the windows first, and a three second deadline ends the
process if a token driver holds up the rest of the teardown.

## Updates, notifications and diagnostics

`updates.rs` checks the latest published release's `latest.json` ten
seconds after launch, hourly, and when the main window opens after fifteen
minutes; `update_install.rs` downloads on the user's click and installs on
Restart now or once idle (no waiting request, no GST connection for five
minutes). `notify.rs` posts local notifications when Settings allows: an
update, a DSC expiring within 30 days (checked daily), a request whose
window lacks focus. `commands/diagnostics.rs` builds the report users copy
into bug reports; nothing is sent.

## Windows and permissions

| Window | Commands it may call |
|---|---|
| `main` | overview, tokens, drivers, doctor, settings, trust, activity, pause, updates, diagnostics |
| `approve` | `get_pending_request`, `refresh_request`, `approve_request`, `cancel_request` |

`build.rs` declares every command; `capabilities/*.json` grants each window
its subset. The CSP allows only the app's own assets and Tauri IPC. The
driver file picker is opened by `add_driver` in Rust, so no window holds a
dialog permission.

## Files on disk

Per-user data directory: `~/Library/Application Support/app.swakshar.desktop`
(shared by the app and the CLI).

| Path | What |
|---|---|
| `tls/ca.pem` | Local CA certificate (trusted in the login keychain) |
| `tls/server.pem`, `tls/server-key.pem` | Loopback leaf and its key (0600) |
| `settings.json` | Preferences |
| `activity.jsonl` | Request history, masked |

Logs: `~/Library/Logs/app.swakshar.desktop/`.

## Local certificate

`swakshar-tls` mints an ECDSA P-256 CA with `basicConstraints` path length 0
and name constraints limited to `127.0.0.1/32` and `localhost`, signs one leaf
valid for 800 days (Apple's limit is 825), and drops the CA key without writing
it. Thirty days before expiry a new pair is minted and must be trusted again.

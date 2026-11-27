# Portal and PDF signing

## Income Tax portal on macOS

The Income Tax section packages the unmodified emBridge 3.2.7.0 installer and
manages its installed launchd service. Choose Install emBridge, complete the
vendor's macOS installation, refresh the status, then choose Start emBridge.
The portal's certificate choice and PIN prompt remain in emBridge.

Start and Stop require native administrator approval. Stop also disables the
service across reboots. Closing Swakshar leaves the companion service alone.
Swakshar independently reports the launchd service, loopback port 26769,
HTTPS trust and the exact loopback hosts alias. It never copies the vendor's
TLS keys or pretends to implement its encrypted portal protocol.

The vendor supplies an x64 macOS companion. Apple Silicon Macs need Apple's
Rosetta. Hardware-specific token drivers still come from their manufacturers;
the bundled PDF runtime does not replace them. Portal service controls and
the bundled vendor installer currently support macOS only.

The owner reported on 2026-11-27 that eMudhra permits redistribution provided
Swakshar is not monetized. This records the owner's report, not an independently
verified public license grant. Revisit this arrangement before monetization.
Private email is not stored in the public repository. emBridge retains its
own ownership and terms; Swakshar's Apache-2.0 source license does not relicense it.

## Sign a PDF

1. Open Sign PDF and choose a PDF under 50 MB.
2. Select any page. Drag to draw the signature rectangle in the page preview.
3. Choose a DSC certificate. Enter the PIN or use the token's PIN pad.
4. Choose Sign and save a copy, then pick a new output filename.

The preview converts through the page's rotation and crop box. Changing pages
clears the old placement. The backend signs an immutable byte snapshot of the
selected PDF, so replacing the source on disk cannot change the document being
signed. Saving uses a private staging directory and an atomic hard link that
cannot replace a file created after the Save dialog. Use a local filesystem
that supports hard links. The original document is not rewritten.

PDF.js, fonts, character maps, image decoding resources, pyHanko, PKCS#11 Python
bindings and the Python interpreter ship in the app. End users install no
Python, pyHanko or command-line tools. The app never searches PATH for a signer.
Each native release builder packages its own runtime; the universal Mac app
carries both arm64 and x64 copies.

The engine creates an offline PAdES signature with the approved DSC CLI vector
appearance: Helvetica, a green check, a Signature valid heading, the certificate
holder and the signing date in UTC, without a heavy border. The heading is part
of the chosen appearance; PDF readers independently validate the signature.
The standard Helvetica appearance supports WinAnsi names; other glyphs use a
replacement character in the appearance, while the signing certificate retains
its complete subject. The preview uses the same font metrics and geometry.
This version
does not add a trusted timestamp, revocation information or long-term validation
data. Password-protected PDFs are refused. It creates a new signature field;
selecting or moving existing form fields is outside this version. The token
certificate and its private key must share a nonempty PKCS#11 object ID.

PDF and GST signatures share the app's single signing slot. The token's latest
lockout flags are read before signing. A failed operation requires an explicit
certificate refresh and never retries a PIN. The PIN is cleared from the form,
held in a Rust secret string, and passed only to the signing child's environment.
It is not put in arguments, configuration, logs or history. The child environment
contains it briefly; software already running as the same user may inspect it.
Process diagnostics are discarded to avoid leaking token identity or credentials.
Token reads stop waiting after twenty seconds, without interrupting a vendor
driver call or submitting a PIN. Refresh after reconnecting an unresponsive token.
PINs with leading or trailing whitespace are refused before any token login,
because the upstream command-line signer trims that input.

Cancel, leaving the editor, quitting and the two-minute deadline terminate the
PDF signing child. Normal success and failure remove staging files. A forced
process termination or power loss can leave its private staging directory for
the owner to remove. emBridge maintains its own separate approval lifecycle.

PDF.js loads only when Sign PDF opens. The native picker returns metadata, then
a main-window-only command transfers that exact snapshot as binary, avoiding
JSON byte arrays. There is no arbitrary-path read API. Parsing and page rendering
show progress, discard obsolete work, and release their worker on editor exit.
Canvas density is capped at two million pixels per visible page. Page controls
stay together and the preview uses a bounded scroll area.

## Build an installer

Run these commands in the Swakshar checkout with Python 3.14.10 available as
`python3` (or set `SWAKSHAR_BUILD_PYTHON` to its full path):

```sh
pnpm install --frozen-lockfile
pnpm bundle:pdf
SWAKSHAR_EMBRIDGE_ARCHIVE="$HOME/Downloads/emBridge-mac-3.2.7.0.zip" pnpm bundle:embridge
pnpm tauri build
```

The emBridge ZIP and DMG must match `scripts/embridge.lock.json`. The current
vendor download may be newer, so it is deliberately not silently substituted.
No vendor binary is checked into git. On native release builders, the repository
variable `SWAKSHAR_EMBRIDGE_ARCHIVE_URL` must point to an HTTPS mirror of that
exact original ZIP. Archive checksums are enforced before extraction. That
mirror is not provisioned by these source changes.

`scripts/pdf-runtime.lock.json` pins every build/runtime package and accepted
artifact hash. All accepted versions were at least seven days old when selected.
Python 3.14.10 was tagged on 2026-10-25, with binaries available for every
native runner in GitHub's Python catalog. PyInstaller packages the upstream
entrypoint without adding project Python or JavaScript source files. Runtime
archives preserve executable permissions and relative symlinks between CI jobs.
Packaging fails if required executables, licenses or the macOS companion are absent.

CI builds the PDF runtimes natively for both Mac processors, Windows x64 and
Linux x64 before the existing Tauri signing and notarization flow. With the
owner's explicit approval, the Mac runtime jobs reuse the existing Developer ID
certificate through Apple's certificate-import action. PyInstaller signs the
helper and collected native libraries with the existing app entitlements before
they enter the bundle. CI refuses an unsigned macOS runtime. The owner still
verifies the final app's signing and notarization before a release.

Cryptography 50 has no Intel Mac wheel. That builder compiles its locked original
source using hash-locked Maturin and static OpenSSL 4.0.3 from
`scripts/openssl.lock.json`. Upstream's Rust lockfile is enforced by its build
configuration. Its license and source digest accompany the frozen runtime;
end users do not need Homebrew or an OpenSSL installation.

## Owner verification

The owner tests stable patch releases through the existing in-app updater.
CI and signed packaging must pass before each release is published as latest.
Real-token, portal and installed-app acceptance then remain with the owner.
Source gates run in CI:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm run check
```

Then verify a fresh installer on a machine without Python or pyHanko: install
emBridge from the bundled DMG; check Start/Stop, trust and portal DSC registration;
sign a PDF on different pages and with rotated/offset page boxes; inspect its
cryptographic signature and visible placement in a PDF reader. Verify wrong-PIN
warnings, cancellation, quitting and protection against replacing existing files.
Check both Mac processor builds and the platform installers during acceptance.

## Upstream references

- [emBridge downloads and platform details](https://embridge.emudhra.com/)
- [pyHanko signing](https://docs.pyhanko.eu/en/stable/cli-guide/signing.html)
- [pyHanko token configuration](https://docs.pyhanko.eu/en/latest/cli-guide/config.html)
- [PDF.js viewport and rendering](https://mozilla.github.io/pdf.js/examples/)
- [PyInstaller packaging](https://pyinstaller.org/en/latest/usage.html)
- [Python 3.14.10 license](third-party/PYTHON.md)

# Testing

## Automated

```sh
cargo test --workspace --locked
pnpm run check
```

| Area | What the tests pin down |
|---|---|
| `protocol` | The portal's exact request frame; reply slicing as the portal's JavaScript does it; greeting bytes; origin and host rules; PAN masking; dates |
| `cms` | Layout (one signer, one certificate, three signed attributes in DER order, attached content); the signer sees exactly the encoded attributes; out-of-range times; base64url; byte-for-byte parity with the reference CMS |
| `token` | CCA fields (PAN hash, classes, key usage) from generated certificates; PAN-first ranking across tokens; filters; Mach-O slice detection; what a plugged-in token still needs; the reply frame matches the reference line for line; only canonical DER certificates are embedded |
| `tls` | Minting, reuse, renewal near expiry, name and basic constraints, key file mode |
| `server` | HTTP routing (origins, host, navigation-only status page); a full greeting, request and reply over an in-memory WebSocket |
| `cli` | The portal's greeting check |

## Reference fixtures

`fixtures/reference` pins Swakshar to the output of an earlier Python signer
that the GST portal accepts. The fixtures were produced by that signer's own
CMS and reply code, with a throwaway identity whose RSA keys existed only in
memory: holder `SWAKSHAR TEST SIGNER`, issuer `Swakshar Test CA 2026`, PAN
`ABCDE1234F`, content `ABCDE1234F`, signing time `2026-09-25T00:00:00Z`.

| File | What |
|---|---|
| `signer-cert.der` | The test certificate: class 2 and 3 policies, signing key usage, the PAN hash as subject serial number |
| `python-signtype1.der` | The reference CMS, with a real RSA signature over the signed attributes |
| `python-reply.txt` | The reference reply frame, exactly as sent to the page |

`cms::parity_tests` rebuilds the CMS from the same certificate, content, time
and signature and asserts identical bytes, and that the signer is handed
exactly the attributes the reference signed. `token::portal_tests` asserts the
reply frame is identical. Because RSA PKCS#1 v1.5 is deterministic, identical
inputs to the token give identical output, so these tests cover everything
except the token call itself.

## Manual acceptance (real token, real portal)

Run in order on Apple Silicon, then on Intel if available.

1. Fresh install, token plugged in, its driver not installed: step 1 names the
   token and says which driver it needs.
2. Install the token's driver: within a few seconds, without restarting
   Swakshar, the guide disappears and steps 1 and 2 complete. The certificate
   appears without a PIN.
3. Install certificate: macOS asks for your password; step 3 completes. Test
   in browser opens `https://127.0.0.1:1585/` without a warning in Chrome and
   Firefox.
4. On the GST portal, register or update the DSC in Chrome. Allow the local
   network prompt. The approval window shows the site, "DSC registration for
   PAN" and the matching certificate first. Sign: the portal succeeds.
5. File a return with DSC. The approval window shows the document fingerprint.
   Sign: the portal issues an ARN.
6. Press Cancel: the portal shows "Signing Cancelled"; Activity shows
   Declined.
7. Enter a wrong PIN once: the window says so and stays open; the token's
   warning appears when it reports one.
8. Plug two tokens: the certificate matching the PAN is preselected.
9. Pull the token during a request: a clear error, nothing signed.
10. Start a request with the token unplugged: the window says no suitable
    certificate. Plug it in and press Check tokens again: it appears.
11. Settings, Add driver file: the picker opens in `/usr/local/lib`. A driver
    for the other processor, or one Swakshar already finds, is refused with a
    reason. Remove takes it off the list.
12. Pause from the menu bar: the portal cannot connect. Resume: it can.
13. Open Swakshar from Applications after setup: the main window appears.
    Open it again while it runs: the window comes forward. With Start at
    login on, logging in keeps it in the menu bar with no window.
14. Repeat 4 to 6 in Edge and Brave.

## Command-line checks

```sh
swakshar doctor                   # drivers, tokens, certificates, trust
swakshar selftest                 # sign "swakshar-selftest" on the token and verify it
swakshar serve                    # in one terminal
swakshar probe --pan ABCDE1234F   # in another: acts exactly like the portal page
```

`probe` tries the portal's ports in order, checks the greeting the way the
portal does, sends the portal's request frame, slices the reply the portal's
way, and verifies the CMS.

## Byte-for-byte comparison with another implementation

RSA PKCS#1 v1.5 is deterministic, so two correct implementations produce
identical CMS bytes for the same token, content and signing time:

```sh
swakshar selftest --content ABCDE1234F --signing-time 1790294400 --out rust.der
```

Produce the other implementation's output for the same content and time, then
`cmp rust.der other.der`.

## A software token for development

SoftHSM stands in for a USB token without real identities:

```sh
brew install softhsm opensc
softhsm2-util --init-token --free --label swakshar-test --pin 1234 --so-pin 12345678
# create an RSA-2048 key and a self-signed test certificate with the same CKA_ID,
# for example with pkcs11-tool --keypairgen and openssl, then:
swakshar doctor --module /opt/homebrew/lib/softhsm/libsofthsm2.so
```

Never commit anything from a real token.

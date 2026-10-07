# Research: covering the vendor-driver tokens

Status: research, 2026-10-07. Companion to `BUNDLED_DRIVERS.md`, which covers
embedding OpenSC. This file is about the four tokens OpenSC does not clearly
support: HYP2003, Watchdata ProxKey, Longmai mToken CryptoID, TrustKey.

Everything here comes from public sources: OpenSC's open-source tree, vendor
websites and the public `usb.ids` list. No vendor driver was read, decompiled
or embedded; `AGENTS.md` clean-room rules forbid that and none of it happened.

## The hard constraint

Only open-source code can be embedded. The vendor drivers for these four are
proprietary, so "read their drivers and embed them" is not open to us: it is
reverse engineering under their licences and banned by `AGENTS.md`. So each
token has exactly three honest outcomes:

1. OpenSC already reads it, so the bundled OpenSC from the other report covers
   it with no new driver.
2. A vendor grants redistribution or publishes a card spec, which turns into
   either a bundled vendor module or a clean-room driver.
3. Neither, so the user installs the vendor driver and the app guides them.

## Route 1: what OpenSC reads today

Checked against a fresh clone of OpenSC `master` (card drivers, ATR tables)
and the public `usb.ids`.

| Token | USB vendor (public) | OpenSC driver | Reads it? |
|---|---|---|---|
| Feitian ePass2003 | `096e:0807`, auto `096e:080a` | `epass2003` (ATR "FTCOS/ePass2003") | Yes, expected |
| SafeNet eToken 5110 | Aladdin `0529` | `idprime` (ATR "eToken 5110") | Yes, expected |
| HYP2003 | Hypersecu `2ccf`; PKI PID not public | none by name | Maybe: see below |
| Longmai mToken CryptoID | not isolated in `usb.ids` | none by name | Maybe: see below |
| Watchdata ProxKey | not a clean CCID entry | none | Unlikely |
| TrustKey | unknown maker | none | Unknown |

Detail per token:

- **HYP2003.** Hypersecu's own macOS guide installs "EnterSafe PKI Manager",
  which is Feitian's EnterSafe stack, and `modules.rs` already maps HYP2003 to
  Feitian's `libcastle`. OpenSC has both an `epass2003` and an `entersafe`
  driver (EnterSafe ATRs "FTCOS/PK-01C", "EJAVA/..."). So there is a real
  chance OpenSC reads a HYP2003 through one of them. Only the token's ATR
  decides it. This is the most promising of the four.
- **Longmai mToken CryptoID.** The vendor brochure says it is a CCID
  "driverless plug and play" device. If that holds, macOS PC/SC sees it with
  no driver, and OpenSC can try its generic ISO 7816 / PKCS#15 path even with
  no model-specific driver. Whether the card's on-token structure is standard
  PKCS#15 is the open question. ATR test needed.
- **Watchdata ProxKey.** No OpenSC driver, and public reports treat it as a
  proprietary USB token needing Watchdata's own driver, not plain CCID. Least
  likely to work through OpenSC.
- **TrustKey.** No public maker or OpenSC entry found. "TrustKey" also names an
  unrelated FIDO vendor, which muddies searches. Treat as unknown until tested.

**The Phase 0 test decides all four.** For each real token, with stock OpenSC:

```sh
opensc-tool --atr                                   # the ATR
opensc-tool --name                                  # driver OpenSC picks, if any
pkcs11-tool --module "$M" --list-slots
pkcs11-tool --module "$M" --login --list-objects --type cert
```

If `--list-objects` shows the certificate, the bundled OpenSC covers that
token and nothing vendor-specific is needed. Record each ATR (not the
certificate contents) so we can add it to a config or an upstream issue.

## Route 2: ask the vendors

A vendor can do one of three useful things: let us redistribute their macOS
PKCS#11 module inside the app, publish the card's command spec so a clean-room
driver is possible, or contribute an OpenSC driver as Feitian did for
ePass2003. Any of these removes the manual install.

Draft outreach for all four is in `docs/research/vendor-outreach.md`. Public
contacts found:

| Vendor | Contact | Note |
|---|---|---|
| Hypersecu (HYP2003) | support@hypersecu.com, info@hypersecu.com | Has a macOS build already; ask about redistribution and an SDK |
| Watchdata (ProxKey) | Bangalore office (Watchdata Technologies India) | No public email found; the issuing CA may route faster |
| Longmai (mToken) | via longmai.net support | Claims CCID; ask if a model-specific OpenSC driver or spec exists |
| TrustKey | via the issuing Certifying Authority or reseller | Maker unconfirmed; the CA knows the real module name |

These are first contacts, not commitments. The owner sends them; I only draft.

## Route 3: clean-room from a published spec

Feasible only where a vendor publishes the card's APDU command set, or the
card follows a public standard (ISO 7816 plus PKCS#15). None of these four
publish a spec today. ePass2003 is the counter-example that already paid off:
Feitian published enough that OpenSC carries a full driver, which is why the
bundled OpenSC covers it. So Route 3 is really a possible outcome of Route 2,
not a separate thing we can start now. We never derive a driver from a vendor
binary.

## Route 4: detect the token and guide the install

When Routes 1 to 3 leave a token needing its vendor driver, the app can still
remove the guesswork: read the connected token's USB vendor and product ID,
name the token, and link its driver page. This needs no vendor code and works
today. It is the guaranteed fallback for ProxKey and anything else OpenSC
cannot read.

- macOS exposes USB VID:PID through IOKit; the PC/SC reader name often carries
  the model too. Build a small table from real tokens (the owner has several),
  since public `usb.ids` is incomplete here: it lists Hypersecu `2ccf` but not
  the HYP2003 PKI PID, and no clean ProxKey entry.
- In `SetupSteps.tsx`, when no certificate is found and a known unsupported
  token is plugged in, show "This looks like a Watchdata ProxKey. Install the
  Watchdata macOS driver, then press Check tokens," with the vendor link.
- This is UI and a data table, no driver code, so it is safe to build before
  any vendor replies.

Built on 2026-10-07: `crates/token/src/usb.rs` reads
`system_profiler SPUSBDataType -json` only when the inventory found no token,
matches public ids and the device's own strings, never reads the serial, and
fills `Inventory.detected` for the Home panel and `swakshar doctor`. Driver
links show as text; an in-app open-link command is a follow-up.

## Where this leaves each token

| Token | Best available path | Needs a vendor driver install? |
|---|---|---|
| ePass2003 | Bundled OpenSC (Route 1) | No, pending real-token check |
| eToken 5110 | Bundled OpenSC (Route 1) | No, pending real-token check |
| HYP2003 | Route 1 if ATR matches EnterSafe/epass2003, else Route 2 | Maybe |
| Longmai mToken | Route 1 if CCID + standard PKCS#15, else Route 2 | Maybe |
| Watchdata ProxKey | Route 4 now; Route 2 for anything better | Yes, for now |
| TrustKey | Route 4 now; identify maker, then Route 1 or 2 | Yes, for now |

## What I did not and will not do

- Did not read, disassemble or embed any vendor driver.
- Did not fabricate USB IDs or ATRs; unknowns are marked unknown.
- Everything embeddable traces to OpenSC's open-source tree.

## Open questions for the owner

1. Run Phase 0 on each real token and record the ATR and whether OpenSC lists
   the certificate. This settles HYP2003, Longmai and TrustKey.
2. Route 4 is built. Verify it with a token whose driver is not installed
   (TESTING.md step 12), and send me the product strings the panel shows so
   the name hints can be tightened.
3. Send the Route 2 drafts? If so, from which address, and do you want the
   HYP2003 one first since Hypersecu already ships macOS support?

## Sources

- OpenSC card drivers (cloned `master`): `src/libopensc/card-epass2003.c`,
  `card-entersafe.c`, `card-idprime.c`, `card-default.c`
- OpenSC supported hardware: <https://github.com/OpenSC/OpenSC/wiki/Supported-hardware-(smart-cards-and-USB-tokens)>
- Public USB IDs: <https://github.com/usbids/usbids>
- Hypersecu India downloads and macOS guide: <https://hypersecu.com/copy-of-downloads-for-india>, <https://hypersecu.com/support>
- Longmai mToken CryptoID brochure (CCID claim): <https://studylib.es/doc/9502671/mtoken-cryptoid-brochure>
- Watchdata India (NPCI vendor listing address): <https://www.npci.org.in/PDF/npci/rupay/approved-vendors/List-of-Vendors-Approved-for-RuPay-Card-Manufacturing-and-Personalization.pdf>
- ProxKey macOS driver reports: <https://developer.apple.com/forums/thread/657046>

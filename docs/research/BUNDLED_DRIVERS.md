# Research: token drivers that work without extra installs

Status: research, 2026-10-07. Nothing here is decided. Bundling a C library
into the signing path needs a `DECISION.md` entry before any code lands.

## Question

Can Swakshar ship what it currently asks users to install with Homebrew, so
the app works straight after install?

## Short answer

The Rust code needs nothing from Homebrew: TLS is rustls with ring, trust uses
`/usr/bin/security`, and PC/SC with a CCID driver is part of macOS. The only
Homebrew item in the user's path is **OpenSC**, the open-source PKCS#11 driver
listed as a fallback in `crates/token/src/modules.rs`.

OpenSC can be bundled. Build it from pinned source in CI as two universal
dylibs with OpenSSL linked statically, put them in `Contents/Frameworks`, sign
them with the app, and list them as the first driver candidate. That covers
**ePass2003** and **SafeNet eToken 5110** with no driver install, pending a
real-token check. Copying Homebrew's build is the wrong route (see below).

Vendor drivers (HYP2003, ProxKey, mToken, TrustKey) cannot be bundled: they
are proprietary, and `AGENTS.md` forbids vendor binaries in the repo. Those
users still install their vendor's driver.

OpenSC's defaults break two Swakshar rules, PIN caching and logout, so the
bundled build needs a small defaults patch. See "Defaults that conflict".

## What a user needs today

| Item | Needed for | Source | Bundle it? |
|---|---|---|---|
| rustls, ring, cryptoki and the rest of the Rust graph | Everything | Compiled in | Already |
| PC/SC and CCID | Talking to USB tokens | macOS (`PCSC.framework`) | Not needed |
| `security` tool | Trusting the local CA | macOS | Not needed |
| Vendor PKCS#11 driver | HYP2003, ProxKey, mToken, TrustKey, older ePass2003 setups | Vendor `.pkg` | No: proprietary |
| OpenSC `opensc-pkcs11.so` | ePass2003, eToken 5110 without a vendor driver | `brew install opensc` or OpenSC `.dmg` | **Yes** |
| SoftHSM, `pkcs11-tool` | Development tests only | Homebrew | No: contributors only |

## Which tokens OpenSC covers

| Token | OpenSC driver | Evidence | Status |
|---|---|---|---|
| Feitian ePass2003 | `epass2003` | ATR table "FTCOS/ePass2003" matches 2022 ATRs from Feitian-initialised cards; CCID lists `096E:0807`, ePass2003Auto `096E:080A` | Expected to work; verify |
| SafeNet eToken 5110, 5110+ FIPS | `idprime` | ATR entries "Gemalto IDPrime MD 940 (eToken 5110)" and "IDPrime 930 (eToken 5110+ FIPS)"; CCID 1.4.36 added 5110+ FIPS | Expected; verify; see context-specific login |
| HYP2003 | Unknown | `modules.rs` maps HYP2003 to Feitian's `libcastle`, a hint it is ePass2003-based; no OpenSC entry found | Test the ATR |
| Watchdata ProxKey, mToken CryptoID, TrustKey | None found | Not in OpenSC's supported list | Vendor driver stays |

## Options

| Option | User installs nothing | Universal | Runs on macOS 12 | Reproducible | Verdict |
|---|---|---|---|---|---|
| A. Keep asking for `brew install opensc` | No | n/a | n/a | n/a | Status quo |
| B. Copy Homebrew's bottle with `dylibbundler` | Yes | No | No | No | Prototype only |
| C. Build pinned OpenSC in CI and bundle | Yes | Yes | Yes | Yes | **Recommended** |
| D. Ask users to run OpenSC's own `.pkg` | No | Likely | Yes | n/a | Fallback docs only |
| E. Rewrite ePass2003 and IDPrime drivers in Rust | Yes | Yes | Yes | Yes | No: secure messaging, years of card quirks, LGPL-derived knowledge |
| F. Bundle vendor drivers | Yes | Mostly x86_64 | Varies | No | Not allowed |

### Why not copy Homebrew's build

- Homebrew's `opensc` depends on `openssl@4` at runtime, so its module loads
  `libcrypto` from `/opt/homebrew/opt/openssl@4`. OpenSC 0.27.1 also needs a
  backported patch for OpenSSL 4.
- A bottle has one architecture. The app ships universal, partly so users with
  x86_64-only vendor drivers can run it under Rosetta, and the bundled module
  must load in both modes.
- A bottle targets the macOS it was built for; the app's minimum is 12.0.
- We would ship a binary we cannot rebuild, and the LGPL source duty still
  applies.

`dylibbundler` (copy every non-system dylib, rewrite install names) is fine for
a one-afternoon prototype on the owner's Mac, not for releases.

## Recommended build

### Inputs

| Input | Version | Why |
|---|---|---|
| OpenSC | 0.27.1 release tarball (2026-03-31) | Latest; 0.27.0 fixed CVE-2025-49010 and CVE-2025-66037 |
| OpenSSL | Latest 3.5.x LTS (3.5.7 in June 2026) | What OpenSC's own macOS build uses; supported to 2030-04-08 |

Pin both by SHA-256 in the build script and check them before extracting.
Both are older than seven days. OpenSC's own script clones the head of the
`openssl-3.5` branch; we use release tarballs instead.

### OpenSSL

Follow OpenSC's `MacOSX/build-openssl-macos.sh`: build `darwin64-arm64` and
`darwin64-x86_64` separately with `no-shared no-module no-apps`, then `lipo`
the two `libcrypto.a` files. `no-module` compiles the providers into
`libcrypto`, which matters because OpenSC loads the legacy provider ("smart
cards depend on several legacy algorithms"); the ePass2003 secure messaging
uses DES-EDE3 and single DES MACs.

### OpenSC

```sh
export MACOSX_DEPLOYMENT_TARGET=12.0
export CFLAGS="-arch arm64 -arch x86_64 -O2" LDFLAGS="-arch arm64 -arch x86_64"
export OPENSSL_CFLAGS="-I$OSSL/include" OPENSSL_LIBS="$OSSL/lib/libcrypto.a"
./configure --prefix="$STAGE" \
  --sysconfdir=/Library/Swakshar/opensc \
  --enable-openssl --enable-sm --enable-zlib --enable-openssl-secure-malloc=65536 \
  --disable-openpace --disable-readline --disable-notify --disable-autostart-items \
  --disable-man --disable-doc --disable-tests --disable-cmocka
make -j4 && make install
```

- The tarball ships `configure`, and the `OPENSSL_*` variables bypass
  pkg-config, so the build needs no Homebrew packages. OpenSC's `MacOSX/build`
  also sets `CRYPTO_CFLAGS` and `CRYPTO_LIBS` to the same values; mirror it.
- `--disable-openpace`: OpenPACE is GPL-3.0 and only serves PACE-based eID
  cards, which Indian DSC tokens are not.
- zlib stays on: the IDPrime driver reads compressed certificates, and macOS
  ships `libz` in `/usr/lib`.
- The Swakshar-specific `sysconfdir` (no spaces, which autotools mishandles)
  keeps the bundled module from reading a system OpenSC config. A missing
  config is not an error in OpenSC.
- PC/SC is opened at run time from `/System/Library/Frameworks/PCSC.framework`.

### Relocation

With shared libraries on, `opensc-pkcs11.so` links `libopensc.<N>.dylib`;
with them off, no PKCS#11 module is built at all. So ship two files:

```sh
cp "$STAGE/lib/opensc-pkcs11.so" out/opensc-pkcs11.dylib
cp "$STAGE/lib/libopensc.$N.dylib" out/
install_name_tool -id "@rpath/libopensc.$N.dylib" "out/libopensc.$N.dylib"
install_name_tool -change "$STAGE/lib/libopensc.$N.dylib" \
  "@loader_path/libopensc.$N.dylib" out/opensc-pkcs11.dylib
```

The rename matters: Tauri's `bundle.macOS.frameworks` accepts only `.dylib`
and `.framework` paths. Then the build fails unless:

| Check | Expect |
|---|---|
| `otool -L out/*.dylib` | Only `@loader_path`, `@rpath`, `/usr/lib` and `/System` |
| `lipo -archs out/*.dylib` | `x86_64 arm64` |
| `vtool -show-build out/opensc-pkcs11.dylib` | `minos 12.0` for both slices |
| `nm -gU out/opensc-pkcs11.dylib` | Only `C_*` symbols (`pkcs11.exports`); OpenSSL stays private |

### Defaults that conflict with Swakshar's rules

| OpenSC default | Effect | Bundled build |
|---|---|---|
| `use_pin_caching = true`, counter 10 | Keeps the PIN in memory for reuse | `false` |
| `C_Logout` clears software PIN state only (`SW_PIN_LOGOUT_ONLY 1`) | The card stays PIN-verified until reset or removal | Keep, and reset on disconnect (next row) |
| PC/SC `disconnect_action = leave` | `C_Finalize` leaves the card verified | `reset`, and Swakshar finalizes the bundled module after each signature |
| `use_file_caching = public` for `idprime` | Writes certificates (holder names) to `~/.cache/opensc` | `no` |

The app cannot pass a config file. The compiled-in path is absolute while the
bundle can move, and `OPENSC_CONF` would need `std::env::set_var`, which is
`unsafe` in edition 2024 under `unsafe_code = "forbid"`. So carry a small
patch to the compiled defaults in `patches/opensc/`, published with the source.

With PIN caching off, keys marked `CKA_ALWAYS_AUTHENTICATE` (common for IDPrime
signature keys) need `C_Login(CKU_CONTEXT_SPECIFIC)` after `C_SignInit`.
`login()` in `crates/token/src/pkcs11.rs` only logs in as `User` today.
cryptoki 0.12.1 has `sign_init`, `sign_update`, `sign_final` and
`UserType::ContextSpecific`, which is enough.

## Bundling in the app

- A release-only overlay, `app/src-tauri/tauri.opensc.json`, lists the two
  dylibs under `bundle.macOS.frameworks`, passed as `tauri build --config`.
  Dev builds keep working without them; a missing framework file fails
  bundling with "Library not found".
- Tauri copies `.dylib` frameworks into `Contents/Frameworks` and signs them
  inside-out before the app. Also sign them in the build job
  (`codesign --timestamp --options runtime`) so a standalone artifact is valid.
- Licence texts go to `Contents/Resources/licenses/` through
  `bundle.resources`.
- `disable-library-validation` stays for vendor drivers. Our dylibs carry our
  team ID, so they would load even if it were removed one day.

## Code changes in Swakshar

1. `modules.rs`: a bundled candidate first, at
   `canonicalize(current_exe())/../../Frameworks/opensc-pkcs11.dylib`, family
   "Built in (OpenSC 0.27.1)". Canonicalising makes it work for a CLI copied
   into `Contents/MacOS` and reached through a symlink in `PATH`.
2. Skip `/Library/OpenSC/...` and `/opt/homebrew/...` when the bundled module
   loads; otherwise the same token shows twice.
3. When a vendor driver and OpenSC both see a token, deduplicate the inventory
   by certificate SHA-256, and sign through one module only.
4. Drop the bundled module's `Pkcs11` context after each signature so
   `C_Finalize` resets the card.
5. Context-specific login for `CKA_ALWAYS_AUTHENTICATE` keys.
6. `swakshar doctor` reports the bundled OpenSC version and slices.
7. Copy in `SetupSteps.tsx` and `README.md` says ePass2003 and eToken 5110
   need no driver, once the owner has verified it on real tokens.

## CI

- New job in `release.yml` (and dispatchable alone) on `macos-15`: download,
  verify hashes, patch, build, relocate, check, sign, upload artifact. Cache it
  with a key of both versions plus the patch hash, so it rebuilds only when
  one changes. The Tauri macOS job downloads it into `vendor/opensc/`
  (ignored by git).
- The steps live in `scripts/build-opensc.ts` (TypeScript only) driving
  `configure` and `make`, under 300 lines. Actions pinned by SHA (G4).
- Never commit the built dylibs.

## Licence duties

| Component | Licence | Duty |
|---|---|---|
| OpenSC | LGPL-2.1-or-later | Ship the licence and notices; offer the exact source we ship, with our patch and build script |
| OpenSSL 3.5 | Apache-2.0 | Ship `LICENSE.txt` |
| OpenPACE | GPL-3.0 | Excluded by `--disable-openpace` |

Attaching the OpenSC tarball, the patch and the build script to every GitHub
release meets the "same place" source offer in LGPL 2.1 sections 4 and 6.
Loading OpenSC as a separate module is a shared-library mechanism, and users
can point Settings at their own OpenSC build without touching the signed
bundle. This is the usual reading, not legal advice.

## Security and upkeep

- OpenSC parses card responses inside our process. Its ePass2003 code has had
  overflows (CVE-2018-16420, fixed in 0.19.0), so exploiting it takes a
  malicious token. Dependabot does not see C sources: add an OpenSC and OpenSSL
  release check to `docs/RELEASING.md`.
- In ePass2003 FIPS mode OpenSC skips the response MAC ("Warning, MAC is not
  checked"). Swakshar already verifies every signature against the
  certificate, which catches a tampered signature. Record this in
  `docs/THREAT_MODEL.md`.
- `OPENSC_DEBUG` in the environment turns on APDU logging, which would show
  the PIN for cards without secure messaging. Only something that already
  controls the user's environment can set it; record it as residual risk.
- Measure the size added to the bundle; a static `libcrypto.a` contributes
  only the object files OpenSC uses.

## Windows and Linux, later

- Windows: OpenSC's own build uses vcpkg `x64-windows-static` for OpenSSL and
  zlib, so `opensc-pkcs11.dll` should be self-contained (check with
  `dumpbin /dependents`). Bundle it as a resource; SignPath signs it.
- Linux: declare `opensc-pkcs11, pcscd` in `bundle.linux.deb.depends` and
  `opensc, pcsc-lite` in `bundle.linux.rpm.depends`. An AppImage cannot ship
  `pcscd`, so it documents it.

## Plan

| Phase | Work | Who |
|---|---|---|
| 0 | Go or no-go on real tokens with stock OpenSC (commands below) | Owner |
| 1 | `DECISION.md` entry; `scripts/build-opensc.ts`, defaults patch, CI job | Agent |
| 2 | Tauri overlay; code changes 1 to 6 | Agent |
| 3 | Licences, release source assets, THREAT_MODEL, RELEASING, TESTING, README | Agent |
| 4 | Windows DLL and Linux package dependencies | Agent, then owner |

Phase 0, for each token family on hand (never commit output from a real
token):

```sh
brew install opensc                    # test only, removed again afterwards
M=/opt/homebrew/lib/opensc-pkcs11.so
opensc-tool --list-readers && opensc-tool --atr
pkcs11-tool --module "$M" --list-slots
pkcs11-tool --module "$M" --list-objects --type cert
pkcs11-tool --module "$M" --list-objects --type privkey --login   # shows always-authenticate
swakshar doctor --module "$M"
swakshar selftest --module "$M"
```

## Open questions

1. Does OpenSC read HYP2003 tokens? (Phase 0 ATR.)
2. Do vendor `libcastle` and OpenSC interfere on one ePass2003 when both are
   loaded in the token thread? (Secure messaging state is per card.)
3. Are eToken 5110 signature keys always-authenticate on Indian DSCs?
4. After `disconnect_action = reset`, does a second process fail to sign
   without a PIN? This is the acceptance test for the logout gap.

## Sources

- OpenSC releases: <https://github.com/OpenSC/OpenSC/releases>
- OpenSC macOS build: <https://github.com/OpenSC/OpenSC/tree/master/MacOSX>
- OpenSC supported hardware: <https://github.com/OpenSC/OpenSC/wiki/Supported-hardware-(smart-cards-and-USB-tokens)>
- ePass2003 driver: <https://github.com/OpenSC/OpenSC/blob/master/src/libopensc/card-epass2003.c>
- IDPrime driver: <https://github.com/OpenSC/OpenSC/blob/master/src/libopensc/card-idprime.c>
- Logout and PIN cache: <https://github.com/OpenSC/OpenSC/blob/master/src/pkcs11/framework-pkcs15.c>, <https://github.com/OpenSC/OpenSC/blob/master/src/libopensc/pkcs15.c>
- PC/SC defaults: <https://github.com/OpenSC/OpenSC/blob/master/src/libopensc/reader-pcsc.c>
- Config loading: <https://github.com/OpenSC/OpenSC/blob/master/src/libopensc/ctx.c>
- Windows build: <https://github.com/OpenSC/OpenSC/blob/master/win32/Make.rules.mak>
- Homebrew formula: <https://github.com/Homebrew/homebrew-core/blob/master/Formula/o/opensc.rb>
- Tauri bundler: <https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/macos/app.rs>
- CCID ePass2003Auto: <https://lists.alioth.debian.org/pipermail/pcsclite-cvs-commit/2015-October/007006.html>
- OpenSSL 3.5 LTS: <https://openssl-library.org/post/2025-02-20-openssl-3.5-lts>
- OpenSSL roadmap: <https://openssl-library.org/roadmap>
- ePass2003 CVE: <https://tenable.com/cve/CVE-2018-16420>
- 0.27.0 CVEs: <https://osv.dev/vulnerability/OESA-2026-2546>

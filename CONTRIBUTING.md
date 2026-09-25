# Contributing

Thank you for helping. Read [AGENTS.md](AGENTS.md) first: it holds the rules
every change follows, for people and tools alike.

## The short version

- Branch from `develop` as `type/topic` (`feat/...`, `fix/...`, `docs/...`).
- One scoped commit per completed unit of work, with a body that says why.
- No AI attribution and no `Co-Authored-By` trailers.
- Keep every file under 300 lines and every function short and documented.
- No `#[allow]`, `@ts-ignore` or lint disables: fix the code instead.

## Development setup

```sh
pnpm install --frozen-lockfile
pnpm run check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm tauri dev
```

## Adding a token driver

Install the vendor's driver, note where the PKCS#11 module lands, and add it to
the list for your OS in `crates/token/src/modules.rs` with the token family.
Say in the pull request which token model and OS you verified it with.

## Provenance

Protocol knowledge must come from public sources: the portal's JavaScript,
frames you captured from your own sessions, and ASN.1 dumps of signatures your
own token produced. Never contribute decompiled vendor code, vendor binaries,
or anything from a real person's token (names, PANs, serials, certificates).

## Reporting a portal change

If the portal starts rejecting signatures, open an issue with the exact portal
message, the date, the browser, and the frames from DevTools (Network, the
`127.0.0.1` row, Messages) with the PAN masked.

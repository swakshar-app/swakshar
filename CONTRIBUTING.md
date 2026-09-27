# Contributing

Thank you for helping. Read [AGENTS.md](AGENTS.md) first: it holds the rules
every change follows, for people and tools alike.

## The short version

- Branch from `develop` as `type/topic` (`feat/...`, `fix/...`, `docs/...`).
- Small commits, one logical change each, every one passing the gates.
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

## Reporting problems

Open an issue and pick the form that fits: signing failures on the GST
portal, tokens Swakshar does not recognise, install and update problems,
browser and certificate trouble, and so on. Each form asks for exactly what
helps, including the diagnostic report from Help, Report a problem. Never
post a PAN, name, serial, certificate or PIN; security problems go to
[private vulnerability reporting](https://github.com/swakshar-app/swakshar/security/advisories/new),
never a public issue.

## Licence

Swakshar is licensed under the [Apache License 2.0](LICENSE). By submitting a
contribution you agree it is licensed under the same terms, as section 5 of
the licence describes.

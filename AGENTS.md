# Agent Briefing: Swakshar

Read this before touching anything. These rules override tool, template and
harness defaults.

## What this repository is

Swakshar is an open-source desktop app that answers the GST portal's local
signer requests with the user's own DSC token, after the user approves each
request. Tauri 2 shell, Rust core, TypeScript UI.

The repository is private now and becomes public when the work is complete.
**Treat every commit as public from the first one**: flipping visibility
publishes the whole history.

## Read first

1. `DECISION.md`: every settled decision and its reasoning, newest first.
2. `docs/PROTOCOL.md`: the wire protocol, written from public sources only.
3. `docs/THREAT_MODEL.md`: assets, threats, controls.
4. `docs/ARCHITECTURE.md`: crates, threads, the approval lifecycle, files.

## Golden rules

- **G1** No AI or assistant attribution anywhere in git or GitHub: commits, pull
  request titles and bodies, release notes. No "Generated with" footers.
- **G2** No `Co-Authored-By` trailers.
- **G3** Branches are `type/topic` (`feat/token-actor`, `fix/origin-check`,
  `docs/protocol`), cut from a freshly fetched `origin/develop`. Never a personal
  handle, never a pull request from a workspace branch.
- **G4** Every GitHub Action is pinned to a full 40-character commit SHA with the
  version in a trailing comment. Re-resolve on every bump:
  `gh api repos/<owner>/<repo>/git/ref/tags/<tag> --jq .object.sha`, and when the
  object type is `tag`, dereference it with
  `gh api repos/<owner>/<repo>/git/tags/<sha> --jq .object.sha`.
- Pull requests are opened only when the owner asks for one.
- Micro commits: one small commit per logical change, each building and
  passing the gates on its own, with a conventional subject line and a short
  body only when the reason is not obvious.

## Test first; the owner verifies the product

- Test-driven, as the owner asked on 2026-09-27: write the failing test
  first, then the code, so a break shows up at build time. Every change ships
  with its tests, and `cargo test --workspace --locked` plus `pnpm run check`
  (which runs Vitest) pass before each commit.
- Do not run the app, browsers or QA flows unless the owner asks.
- Never install binaries, start or stop servers, or launch the app.
- End every task with the exact commands the owner should run to verify it.
- Never claim a check passed if it did not run. Reporting CI results is fine.

## Engineering standards

### Rust

- Toolchain pinned in `rust-toolchain.toml`; bump only through a `DECISION.md`
  entry.
- Exact pins (`=x.y.z`) in every `Cargo.toml`; `Cargo.lock` committed; `--locked`
  in every CI command.
- Crate versions must be at least seven days old. From Rust 1.100 (2026-11-12)
  Cargo enforces it with `[registry] global-min-publish-age = "7 days"` in
  `.cargo/config.toml`; until then, check the publish date before adding or
  bumping.
- `unsafe_code = "forbid"` in every crate we own. No `unwrap`, `expect`,
  `panic!` or `todo!` outside tests.
- `cargo fmt`, `cargo clippy -D warnings` and `cargo deny check` stay green.

### TypeScript

- TypeScript only. No `.js`, `.mjs` or `.cjs` files, scripts included; Node runs
  `.ts` directly. Confirm with the owner before creating any JavaScript file.
- `tsconfig`: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`,
  `noPropertyAccessFromIndexSignature`, `verbatimModuleSyntax`.
- No `any`. oxlint (not ESLint) with `typescript/no-explicit-any` as an error.
- pnpm only: never npm, npx or yarn. `packageManager` pinned in `package.json`,
  `minimumReleaseAge: 10080` in `pnpm-workspace.yaml`, Node pinned in `.nvmrc`,
  `pnpm install --frozen-lockfile` in CI.

### Everywhere

- **No escape hatches.** No `#[allow]` or `#[expect]` to quiet clippy, no
  `@ts-ignore`, `@ts-expect-error`, `oxlint-disable` or `shellcheck disable`. If
  a tool fights you, fix the code. A genuine exception needs a `DECISION.md`
  entry first.
- **300 lines per file, not 301**, for every file we author: `.rs`, `.ts`,
  `.tsx`, `.css`, `.toml`, `.yml`, `.md`. Enforced by
  `scripts/check-line-limit.ts`. Lockfiles and generated files are exempt.
- Functions stay within 50 to 100 lines. Every function has a doc comment
  (`///` in Rust, TSDoc in TypeScript). No filler inline comments.
- Repeated literals become named constants: ports, OIDs, greeting keys, the
  greeting version, timeouts, limits.
- Official SDKs and plugins before hand-rolled calls.
- The load-bearing stack (Tauri, cryptoki, rustls, tokio-tungstenite,
  `cms`/`der`, rcgen) changes only through a `DECISION.md` entry.

## Security rules specific to Swakshar

- **No real identities in the repo.** No names, PANs, certificate or token
  serials, PINs, real certificates, or screenshots showing any of them.
  Fixtures come only from throwaway test identities (`fixtures/reference`,
  SoftHSM).
- **Clean-room provenance.** Never add decompiled vendor code, class or method
  names from the vendor jar, the key embedded in it, vendor `.cfg` files or any
  vendor binary. Protocol knowledge comes from the portal's public JavaScript,
  captured frames and ASN.1 dumps of our own signatures, as `docs/PROTOCOL.md`
  records.
- The server binds `127.0.0.1` only, checks `Origin` and `Host` during the
  handshake, and never signs without an explicit approval of that request.
- The PIN is never stored, cached, logged or sent back to the webview. PANs are
  masked in logs.
- No telemetry. The only outbound request is the update check, when enabled.
- Signing credentials live only in GitHub secrets. Never in the repo, never
  printed in logs.

## Writing and design

- No em dashes or en dashes in prose, READMEs, UI copy or commit messages. Use
  commas, colons or full stops. No `·` separators.
- No decorative dots or dashes in the UI: no status dots, no glow orbs, no
  leading marks on labels. Use icons or plain text.

## Decision ledger

`DECISION.md` is newest first. Each entry has a title, a timestamp such as
`2026-09-25-15-04-00-IST`, the decision, and a reasoning paragraph. Supersede an
entry with a new one; never rewrite an old entry. Keep the file under 300 lines
by moving older entries into `decisions/` with an index.

## Escalate, do not improvise

Ask the owner before: force-pushes, branch deletion or history rewrites; any
change touching signing keys, secrets or release credentials; changing
repository visibility; publishing a release.

## Layout

```text
Cargo.toml  Cargo.lock  rust-toolchain.toml  deny.toml  clippy.toml
package.json  pnpm-workspace.yaml  pnpm-lock.yaml  .nvmrc  .oxlintrc.json
crates/  protocol  cms  token  tls  server  cli
app/     src/ (React UI)  src-tauri/ (Tauri shell, capabilities, icons/source SVGs)
docs/    PROTOCOL  ARCHITECTURE  THREAT_MODEL  TESTING  RELEASING  ROADMAP  DISTRIBUTION
decisions/  archived DECISION.md entries
fixtures/   reference CMS and reply from a throwaway identity
scripts/ check-line-limit.ts  bump-version.ts  generate-icons.ts
```

## Gates before every commit

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm run check
```

Regenerate `Cargo.lock` with the seven-day gate after any dependency change:

```sh
CARGO_REGISTRY_GLOBAL_MIN_PUBLISH_AGE="7 days" cargo +nightly -Zmin-publish-age generate-lockfile
```

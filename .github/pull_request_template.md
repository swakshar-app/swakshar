## What and why

<!-- What this changes and why. Link the issue it closes, for example "Closes #12". -->

## How it was tested

<!-- The failing test written first, the gates run, and anything checked by
hand (token model, macOS version, browser). -->

## Checklist

- [ ] The tests came first, and `cargo test --workspace --locked` and `pnpm run check` pass
- [ ] `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings` pass
- [ ] Small commits, one logical change each, with conventional subjects
- [ ] No real identities anywhere: no PAN, name, token or certificate serial, certificate or PIN in code, fixtures, screenshots or this description
- [ ] No vendor code, vendor binaries or decompiled material; protocol knowledge comes from public sources, as `docs/PROTOCOL.md` records
- [ ] No AI or assistant attribution in commits or this description
- [ ] New dependencies are at least seven days old and pinned exactly
- [ ] `CHANGELOG.md` has an entry for anything a user will notice
- [ ] `DECISION.md` has an entry for anything that changes a settled decision

# Dependency Sweep Automation

The dependency sweep maintains dependency pins that Dependabot does not reliably
update. It must read [the automation conventions](README.md), this runbook, and
[the dependency posture](../dependencies.md) before acting.

Run weekly. Review human feedback from prior runs before repeating the same
class of change.

## Scope

The sweep owns:

- exact non-Rust tool versions in `mise.toml`, including Node.js, Ruby, Zizmor,
  and cargo-installed development tools;
- the pnpm `packageManager` pin in `package.json`;
- pnpm lockfile refreshes and transitive Node updates;
- `pnpm-workspace.yaml` hardening settings;
- documentation required to keep this posture accurate.

Dependabot owns Rust crate updates, direct Node dependency bumps, and GitHub
Actions pins. The Unicode automation owns Unicode data, its license, generated
Rust tables, and Unicode-driven crate releases.

## Rules

- Pin non-Rust tools exactly. Rust remains on the `stable` channel.
- Apply a one-week cooldown to new toolchain and package-manager releases.
- Keep pnpm files toolchain-managed; do not format them with Prettier.
- Update manifests and lockfiles together.
- Use authoritative upstream releases and compare the exact old and new refs.

Start each run by reviewing open Dependabot pull requests. Inspect the upstream
diff—not only Dependabot's summary—before merging or enabling auto-merge.
Generated actions require matching source changes, and updates that alter
runtime requirements, downloads, cache behavior, PATH handling, or post-action
behavior require human review.

Low-risk, understood, green Dependabot updates may be merged or placed on
auto-merge. Leave ambiguous, high-risk, or insufficiently reviewable updates
open with a `Codex automation note:` explaining the blocker and upstream compare
reviewed.

## Pull Requests

Use one branch and pull request per logical dependency domain. Apply `A-deps`,
`C-automation`, and `codex`. Summarize old and new versions, upstream
comparisons, cooldown decisions, risk classification, validation, and Dependabot
PR disposition.

Run:

```sh
mise run fmt
mise run lint
mise run test
cargo package --allow-dirty
```

Open an inbox item after every run, including when no change is needed.

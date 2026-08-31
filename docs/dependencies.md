# Dependency and Supply-Chain Posture

Roe keeps dependency surfaces small, pinned, and auditable. Every dependency
should have an owner, a reason to exist, and a reproducible update path.

## Principles

- Prefer Rust standard library code and repository-local helpers when behavior
  is small, stable, and easy to test.
- Add dependencies only when they reduce more risk or complexity than they
  introduce.
- Pin direct development tooling dependencies exactly.
- Do not use mutable selectors such as `latest`, `lts`, branch names, broad
  version ranges, or floating tags for durable tooling inputs.
- Rust is the explicit exception: this Rust library intentionally tracks the
  `stable` channel in `mise.toml`.
- Apply a one-week cooldown before adopting newly released toolchain and
  package-manager versions.
- Keep generated artifacts reproducible from repository-owned scripts and
  authoritative upstream inputs.
- Keep package-manager lockfiles owned by their package managers. Prettier must
  not rewrite `pnpm-lock.yaml` or `pnpm-workspace.yaml`.

## Ownership

- Dependabot owns Rust crate updates, direct Node dependency updates, and pinned
  GitHub Actions.
- The dependency-sweep automation owns non-Rust `mise.toml` tools, the pnpm
  package-manager pin, pnpm lockfile refreshes, and workspace hardening.
- The Unicode maintenance automation owns Unicode case-mapping data, the Unicode
  license, and generated mapping tables.
- Maintainers own dependency-policy changes and release decisions outside the
  explicit Unicode versioning policy.

## Toolchains

`mise.toml` is the canonical local toolchain manifest. Node.js, Ruby, Zizmor,
and cargo-installed development tools must be pinned to exact versions. Rust
tracks `stable` by design.

Node.js should track the latest LTS line after cooldown. Local bootstrap uses
Corepack to select pnpm from the exact `package.json#packageManager` pin. CI
uses the pinned `pnpm/action-setup` action.

Node dependencies are limited to deterministic text formatting. Rust
dependencies should stay close to the standard library unless a dependency is
clearly justified by crate behavior.

## Enforcement

Dependency changes must preserve this posture, update generated or lockfile
artifacts with their owning tools, and pass relevant formatting, linting, tests,
and package validation before merge.

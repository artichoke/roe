# Unicode Maintenance Automation

This monthly automation checks for a new stable Unicode Character Database
release and keeps Roe's embedded data current. It must read
[the automation conventions](README.md), this runbook, and
[the dependency posture](../dependencies.md) before acting.

## Owned Inputs

- `UnicodeData.txt`, `SpecialCasing.txt`, and `PropList.txt` from the latest
  Unicode Character Database;
- `LICENSE-UNICODE` from the Unicode license endpoint;
- `generated/case_mapping.rs`.

Use Unicode's authoritative versioned data and license endpoints. Do not use
mirrors, release candidates, beta data, or mutable third-party copies. Preserve
the Unicode Terms of Use commentary in `scripts/update_unicode.rb`.

## No-Change Runs

Compare the embedded file headers and generated data with the latest stable
Unicode release. If already current, make no git changes and open an inbox item
with the checked Unicode version and source links.

## Update Runs

Use the repository tasks:

```sh
mise run unicode:update
git add LICENSE-UNICODE generated/ucd
mise run unicode:build
mise run fmt
mise run lint
mise run test
cargo package --allow-dirty
```

Review every generated diff. Confirm the downloaded license is present in the
crate package and that generated tables are deterministic on a second build. The
explicit staging step is required by the generator's clean-input guard; stage
`generated/case_mapping.rs` after reviewing it.

While Roe is on `0.0.x`, a Unicode data update prepares the next patch release
unless a maintainer directs otherwise. Update `Cargo.toml`, README dependency
examples and Unicode prose, and `html_root_url` together. Do not create tags,
publish crates, or create GitHub releases.

Open one pull request with `A-unicode`, `A-release`, `C-automation`, and
`codex`. Include the old and new Unicode versions, authoritative source links,
generated files changed, version-bump rationale, and validation results.

Unicode updates change behavior and must never be merged or placed on auto-merge
by this automation. Open an inbox item requesting maintainer review.

# GitHub Actions Runner Images Automation

This weekly automation keeps Roe workflow runner labels aligned with
GitHub-hosted runner support. It must read
[the automation conventions](README.md) and this runbook before acting.

## Scope

Review `.github/workflows/`, especially `runs-on` labels and assumptions that
vary by image. Preserve current generally available Ubuntu, Windows, and macOS
coverage. Jobs with stable coverage requirements should use explicit maintained
labels instead of moving `*-latest` aliases.

Dependabot owns GitHub Actions versions; this automation changes runner labels
only.

## Sources

Use current authoritative sources:

- the `actions/runner-images` supported-images table;
- open `actions/runner-images` issues labeled `Announcement`;
- GitHub-hosted runner documentation;
- recent Roe CI runs.

Report concrete migration, deprecation, brownout, and retirement dates.

## Changes

If no change is needed, create no branch or pull request. Open an inbox item
listing the labels and sources reviewed.

When labels must change, keep the diff scoped to runner maintenance and inspect
the repository's branch rulesets:

```sh
gh ruleset list --repo artichoke/roe
gh api repos/artichoke/roe/rulesets/RULESET_ID \
  --jq '.rules[] | select(.type == "required_status_checks").parameters.required_status_checks[].context'
```

Update required check contexts when matrix job names change. If permissions
prevent a ruleset update, open the pull request and lead the inbox item with the
exact manual change required.

Apply `A-build`, `C-automation`, and `codex`. Do not enable auto-merge for
preview images, ambiguous announcements, unexplained CI failures, or removal of
an OS family. Low-risk mechanical moves between generally available images may
use auto-merge after validation.

Run:

```sh
git diff --check
mise run fmt:check:text
```

Run actionlint when available. Open an inbox item after every run.

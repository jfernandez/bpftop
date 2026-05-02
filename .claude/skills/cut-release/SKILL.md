---
name: cut-release
description: Cut a new versioned release of bpftop — pick the version, open a version-bump PR, sign-tag the merge commit on main, and draft GitHub release notes in the project's established format. Use whenever the user says "cut a release", "release X.Y.Z", "ship X.Y.Z", "time to release", "tag a new version", "bump to X.Y.Z", "let's do a release", or otherwise initiates a release. Trigger even when the user only asks for one step ("what version should we bump to?", "make the release PR", "tag it") — load this skill so the broader workflow context is available, and only do the step the user asked for.
---

# Cut a release

This is the bpftop release runbook. Five steps: pick the version, open the bump PR, wait for merge, push a signed tag, fill in the release notes.

Tags and releases are user-visible and permanent. Confirm the version, the tag SHA, and the release notes content with the user before pushing the tag and before publishing the draft.

---

## Step 1: Pick the version

```bash
git log $(git describe --tags --abbrev=0)..HEAD --oneline
```

bpftop is pre-1.0. Apply semver:

- **Patch** (`0.x.y` → `0.x.(y+1)`) — only fixes, refactors, dep bumps
- **Minor** (`0.x` → `0.(x+1)`) — any new user-facing feature, or any breaking change to CLI flags / TUI keybindings / output format

CI churn, AI-assistant config tweaks, and dependabot bumps that don't change runtime behavior are not on their own a reason to cut a release.

Tell the user which commits drove the decision and let them confirm or override.

### When the user asks about going to 1.0

The signal is UX/API stability, not feature completeness. Recommend 1.0 once the user is willing to commit that the next release won't break CLI flags or TUI keybindings without a deprecation cycle, the feature surface is additive (no "initial X" support, no churning subsystems), and they'd be comfortable with a distro packaging it. When something just landed in the prior release labeled "initial", recommend one more minor first.

---

## Step 2: Open the bump PR

```bash
git checkout -b release/X.Y.Z
```

Edit `Cargo.toml`:

```toml
[package]
name = "bpftop"
version = "X.Y.Z"
```

Refresh the lockfile:

```bash
cargo update -p bpftop --precise X.Y.Z
```

The diff should be exactly two lines: one in `Cargo.toml`, one in `Cargo.lock`. Verify before committing.

Commit format is strict and minimal — subject only, **no body**, with a `Signed-off-by` trailer:

```bash
git commit -s -m "chore: bump version to X.Y.Z"
```

No commit body. Release notes live on the GitHub release page, not in the bump commit. This is a project convention — past releases (e.g. v0.8.0 commit `aab2d53`) follow this minimal pattern, and duplicating context here is noise.

Push and open the PR with an **empty body**:

```bash
git push -u origin release/X.Y.Z
gh pr create --title "chore: bump version to X.Y.Z" --body ""
```

Do not merge the PR. Wait for the user to merge it.

---

## Step 3: Wait for merge and green CI

```bash
gh pr view <N> --json state,mergeCommit,statusCheckRollup
```

Look for `state: MERGED` and all checks `SUCCESS` (`build_and_test` on x86_64 and aarch64). Capture `mergeCommit.oid` — that's what gets tagged.

---

## Step 4: Cut the signed annotated tag

```bash
git checkout main && git pull --ff-only origin main
git tag -s vX.Y.Z -m "vX.Y.Z" <merge-commit-sha>
git tag -v vX.Y.Z
git push origin vX.Y.Z
```

The tag message body is the bare string `vX.Y.Z` — no "Release ..." prefix, no prose. Matches the v0.8.0 style.

After pushing, CI's `create_release` job auto-creates a **draft** release with the cross-compiled binaries (`bpftop-aarch64-unknown-linux-gnu`, `bpftop-x86_64-unknown-linux-gnu`) attached. Wait a few seconds, then verify:

```bash
gh release list --limit 3
gh release view vX.Y.Z
```

You'll edit this draft in step 5. Don't `gh release create` a second one.

---

## Step 5: Draft the release notes

The template lives at `assets/release-notes-template.md` and a worked example for v0.9.0 is at `assets/release-notes-example-v0.9.0.md`. Read both before drafting — the example is the single best reference for tone, grouping, and formatting.

The structure, in order:

1. `## What's New` — one short prose paragraph (2-3 sentences) summarizing the headline changes
2. `### Features` — `feat:` commits, new user-facing functionality
3. `### Fixes` — `fix:` commits
4. `### Maintenance` — `chore:`, `refactor:`, `ci:`, dep bumps. Drop internal AI-assistant / Claude-workflow churn — invisible to users, clutters the changelog.
5. `## New Contributors` — first-timers, one line each
6. `## Contributors` — alphabetical `Display Name @login`, one per line
7. Full Changelog link — `https://github.com/jfernandez/bpftop/compare/vPREV...vX.Y.Z`

Each Features/Fixes/Maintenance bullet ends with `(#N)` referencing its PR. No `by @author` inline — attribution lives in the dedicated Contributors section. Direct-to-main commits with no PR don't get a number.

### Resolving the Contributors and New Contributors sections

Run the bundled script. It encodes the project's exclusion and sort rules, and resolves first-timer status correctly via PR history:

```bash
.claude/skills/cut-release/scripts/contributors.sh vPREV
```

Output is two ready-to-paste markdown sections.

The script enforces:

- **First-timer detection via PR history per GitHub login.** Comparing commit author names (`git log --pretty=%an`) misses people whose local `git config user.name` has changed since their first contribution — the same person can appear under different names across years. The login is stable.
- **Bots excluded.** GitHub reports bots as either `dependabot[bot]` or `app/dependabot`; the script filters both forms.
- **Maintainer (`jfernandez`) included.** This is a deliberate choice — the user wants their own name in the Contributors list. The convention in some projects is to omit the publisher; bpftop chooses to include them.
- **Alphabetical by display name, case-insensitive.**

### Apply the notes and link the user

```bash
gh release edit vX.Y.Z --notes "$(cat <<'EOF'
...notes...
EOF
)"
```

Tell the user the URL of the draft and wait for review. Publish only after they confirm:

```bash
gh release edit vX.Y.Z --draft=false
```

---

## Things that bit us before

- **GitHub shows the GPG key as expired but it isn't locally.** Extending a GPG key's expiration locally doesn't propagate to GitHub, and same-fingerprint re-uploads are silently no-op'd. Fix: delete the existing GPG key on GitHub Settings → SSH and GPG keys, then re-upload `gpg --armor --export <keyid>`. Past commits stay verified once re-uploaded.
- **Mistakenly flagged a returning contributor as a first-timer.** Always use the bundled `contributors.sh` script — don't eyeball it from `git log`.
- **Slipped a body into the release commit.** If a body sneaks in, `git commit --amend` and force-push the branch before merging. Subject only + sign-off, empty PR body.
- **Tagged the local feature-branch tip instead of the merge commit.** Always `git checkout main && git pull --ff-only` before tagging, then tag the merge commit by SHA captured from `gh pr view`.
- **Created a second draft when CI already auto-created one.** CI creates the draft on tag push. Always `gh release edit vX.Y.Z` to populate it; never `gh release create`.

---

## Doing only one step

If the user says "just tag it" or "just draft the notes", do only that step. Still confirm version / tag SHA / notes content before any irreversible action (push, publish).

---

## File layout

```
.claude/skills/cut-release/
├── SKILL.md                                    (this file — workflow)
├── assets/
│   ├── release-notes-template.md               (literal template with placeholders)
│   └── release-notes-example-v0.9.0.md         (worked example — best reference)
└── scripts/
    └── contributors.sh                         (resolves Contributors + New Contributors)
```

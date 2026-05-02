#!/usr/bin/env bash
# Resolve "New Contributors" and "Contributors" lists for a bpftop release.
#
# Usage: contributors.sh <prev-tag>
# Example: contributors.sh v0.8.0
#
# Output: two markdown sections, ready to paste into the release notes:
#   ## New Contributors
#   ## Contributors
#
# Conventions encoded:
#   - Source of truth is PR history per GitHub login (not commit author names).
#   - Bots (logins ending in [bot]) are excluded from both lists.
#   - The maintainer (jfernandez) is included in Contributors.
#   - Contributors are sorted alphabetically by display name, case-insensitive.

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <prev-tag>" >&2
  exit 2
fi

PREV_TAG="$1"
REPO="jfernandez/bpftop"

# ISO date of the previous tag (use commit date — annotated-tag dates are tag creation, not commit).
PREV_DATE=$(git log -1 --format=%cI "${PREV_TAG}")

# All PRs merged after the previous tag, JSON.
PRS_JSON=$(gh pr list \
  --state merged --base main \
  --search "merged:>=${PREV_DATE}" \
  --json number,author,mergedAt \
  --limit 100)

# Unique non-bot author logins, sorted.
# `gh pr list` reports bots as either "name[bot]" or "app/name" — exclude both.
LOGINS=$(echo "${PRS_JSON}" | jq -r '
  .[] | .author.login
  | select(endswith("[bot]") | not)
  | select(startswith("app/") | not)
' | sort -u)

if [[ -z "${LOGINS}" ]]; then
  echo "## New Contributors"
  echo "_(none)_"
  echo
  echo "## Contributors"
  echo "_(none)_"
  exit 0
fi

# First-timers: a login whose earliest-ever merged PR in this repo falls within this window.
declare -A IN_WINDOW
while read -r number; do
  IN_WINDOW["${number}"]=1
done < <(echo "${PRS_JSON}" | jq -r '.[].number')

NEW_CONTRIBUTORS=()
declare -A FIRST_PR_IN_WINDOW
while read -r login; do
  [[ -z "${login}" ]] && continue
  first_pr=$(gh pr list --author "${login}" --state merged \
    --json number,mergedAt --limit 100 \
    | jq -r 'sort_by(.mergedAt) | .[0].number // empty')
  if [[ -n "${first_pr}" && -n "${IN_WINDOW[${first_pr}]:-}" ]]; then
    NEW_CONTRIBUTORS+=("${login}")
    FIRST_PR_IN_WINDOW["${login}"]="${first_pr}"
  fi
done <<< "${LOGINS}"

# Display names — fall back to login if `name` is empty.
declare -A DISPLAY_NAME
while read -r login; do
  [[ -z "${login}" ]] && continue
  name=$(gh api "users/${login}" --jq '.name // ""')
  if [[ -z "${name}" ]]; then
    name="${login}"
  fi
  DISPLAY_NAME["${login}"]="${name}"
done <<< "${LOGINS}"

# Emit New Contributors.
echo "## New Contributors"
if [[ ${#NEW_CONTRIBUTORS[@]} -eq 0 ]]; then
  echo "_(none)_"
else
  for login in "${NEW_CONTRIBUTORS[@]}"; do
    pr="${FIRST_PR_IN_WINDOW[${login}]}"
    echo "* @${login} made their first contribution in https://github.com/${REPO}/pull/${pr}"
  done
fi
echo

# Emit Contributors — sorted alphabetically by display name, case-insensitive.
echo "## Contributors"
{
  while read -r login; do
    [[ -z "${login}" ]] && continue
    printf '%s\t%s\n' "${DISPLAY_NAME[${login}]}" "${login}"
  done <<< "${LOGINS}"
} | sort -f | awk -F'\t' '{ printf "* %s @%s\n", $1, $2 }'

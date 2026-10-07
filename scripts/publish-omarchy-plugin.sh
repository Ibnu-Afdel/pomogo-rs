#!/usr/bin/env bash
# Publish contrib/omarchy/plugin to its standalone repository so Omarchy
# users can `omarchy plugin add` it. The plugin's history is split out of
# this repository, so the standalone repo always fast-forwards.
#
#   scripts/publish-omarchy-plugin.sh [remote-url]
set -euo pipefail

REMOTE="${1:-git@github.com:Ibnu-Afdel/omarchy-pomogo.git}"

cd "$(git rev-parse --show-toplevel)"

if command -v omarchy-plugin-validate >/dev/null; then
  omarchy-plugin-validate contrib/omarchy/plugin
fi

commit=$(git subtree split --prefix=contrib/omarchy/plugin HEAD)
git push "$REMOTE" "$commit:refs/heads/main"
echo "Published $commit to $REMOTE"

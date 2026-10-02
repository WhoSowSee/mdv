#!/usr/bin/env bash
set -euo pipefail

tag=${GITHUB_REF#refs/tags/}
version=${tag#v}
commit=$(git rev-list -n 1 "$tag")
tag_message=$(git for-each-ref "refs/tags/$tag" --format='%(contents)')
commit_message=$(git show -s --format=%B "$commit")
changelog_section=""
if [ -f CHANGELOG.md ]; then
  changelog_section=$(awk -v ver="$version" '
    function trim(str) {
      sub(/^[[:space:]]+/, "", str)
      sub(/[[:space:]]+$/, "", str)
      return str
    }
    /^## \[/ {
      if (match($0, /^## \[([^\]]+)\]/, m)) {
        heading = trim(m[1])
        if (found && heading != ver) exit
        if (!found && heading == ver) {
          found = 1
          next
        }
      }
    }
    found {
      if (!printed && $0 ~ /^[[:space:]]*$/) next
      printed = 1
      print
    }
  ' CHANGELOG.md)
fi
if [ -n "${changelog_section//[$'\r\n\t ']/}" ]; then
  message=$changelog_section
elif [ -n "${tag_message//[$'\r\n\t ']/}" ]; then
  message=$tag_message
else
  message=$commit_message
fi

printf '%s\n\n%s' "$message" '---' > release-notes.md

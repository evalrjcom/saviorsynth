#!/usr/bin/env bash
# Usage: tools/bump_version.sh <new-number>     e.g.  tools/bump_version.sh 2   (saviorsynth_v5 -> saviorsynth_v5)
# Updates EVERY in-tree reference (Cargo.toml name+version, docs, scripts, doc file names) in one go so nothing is left stale.
set -euo pipefail
cd "$(dirname "$0")/.."
NEW="${1:?new version number required}"
[[ "$NEW" =~ ^[0-9]+$ ]] || { echo "version must be an integer"; exit 2; }
OLD_ID=$(sed -n 's/^name *= *"\(.*\)"/\1/p' Cargo.toml | head -1)
OLD_N="${OLD_ID##*_v}"
NEW_ID="${OLD_ID%_v*}_v${NEW}"
echo "$OLD_ID -> $NEW_ID"
FILES=$(grep -rIl --exclude-dir=target --exclude-dir=dist --exclude-dir=.git "$OLD_ID" . || true)
for f in $FILES; do sed -i "s/$OLD_ID/$NEW_ID/g" "$f"; done
sed -i "s/^version *= *\"[0-9]*\.\([0-9]*\)\.\([0-9]*\)\"/version = \"${NEW}.0.0\"/" Cargo.toml
for f in $(find . -path ./target -prune -o -path ./dist -prune -o -name "*${OLD_ID}*" -print); do
  mv "$f" "${f//$OLD_ID/$NEW_ID}"
done
rm -f Cargo.lock.bak
cargo generate-lockfile >/dev/null 2>&1 || true
echo "done. Rename the project folder to $NEW_ID if it still carries the old name."

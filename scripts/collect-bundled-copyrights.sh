#!/usr/bin/env bash
# Collects the real copyright/license text for every native library bundled into a Linux
# AppImage build, from dpkg's own machine-readable records on the build runner.
#
#   ./scripts/collect-bundled-copyrights.sh <AppDir path> <output file>
#
# The AppImage is self-contained (unlike the .deb/.rpm, which declare these as system
# dependencies and bundle nothing): it physically copies ~130 shared libraries - GTK,
# WebKitGTK, GnuTLS, GStreamer, and more - many LGPL-2.1(+). Redistributing them is
# permitted but the LGPL expects their copyright notices and license text to travel with
# the distribution, not just a pointer to where they *could* be found. This runs on the
# same runner that built the AppImage (see .github/workflows/release.yml), so it reads
# each bundled library's copyright straight from the exact package version that produced
# it - accurate per build, no hand-maintained list to fall out of sync.
set -euo pipefail

APPDIR="${1:?usage: collect-bundled-copyrights.sh <AppDir path> <output file>}"
OUT="${2:?usage: collect-bundled-copyrights.sh <AppDir path> <output file>}"

if [ ! -d "$APPDIR/usr/lib" ]; then
  echo "error: $APPDIR/usr/lib not found" >&2
  exit 1
fi

# Build a basename -> installed-path index once, rather than re-walking /usr/lib per
# bundled file (there are ~130 of them).
index_file="$(mktemp)"
trap 'rm -f "$index_file"' EXIT
find /usr/lib /lib -type f \( -name '*.so' -o -name '*.so.*' \) 2>/dev/null \
  | awk -F/ '{print $NF, $0}' > "$index_file"

{
  echo "================================================================================"
  echo "LINUX BUNDLED LIBRARY COPYRIGHTS (this build's AppImage)"
  echo "================================================================================"
  echo
  echo "The AppImage bundles the native libraries below. This file is generated fresh on"
  echo "every release build directly from dpkg's copyright records for the exact package"
  echo "versions installed on the runner that built it - see"
  echo "scripts/collect-bundled-copyrights.sh."
  echo
} > "$OUT"

declare -A pkg_seen
shopt -s nullglob
for so in "$APPDIR"/usr/lib/*.so*; do
  name="$(basename "$so")"
  orig="$(awk -v n="$name" '$1 == n { print $2; exit }' "$index_file")"
  [ -z "$orig" ] && continue

  pkg="$(dpkg -S "$orig" 2>/dev/null | head -1 | cut -d: -f1)" || true
  [ -z "${pkg:-}" ] && continue
  [ -n "${pkg_seen[$pkg]:-}" ] && continue
  pkg_seen["$pkg"]=1

  version="$(dpkg-query -W -f='${Version}' "$pkg" 2>/dev/null || echo unknown)"
  copyright="/usr/share/doc/$pkg/copyright"

  {
    echo "--------------------------------------------------------------------------------"
    echo "$pkg $version"
    echo "--------------------------------------------------------------------------------"
    echo
    if [ -f "$copyright" ]; then
      cat "$copyright"
    else
      echo "[No /usr/share/doc/$pkg/copyright found on the build runner for this package.]"
    fi
    echo
  } >> "$OUT"
done

count="${#pkg_seen[@]}"
echo "Wrote $OUT ($count packages)" >&2

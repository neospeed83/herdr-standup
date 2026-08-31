#!/bin/sh
set -eu
version=0.4.0
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) asset=herdr-standup-macos-aarch64 ;;
  Darwin-x86_64) asset=herdr-standup-macos-x86_64 ;;
  Linux-aarch64|Linux-arm64) asset=herdr-standup-linux-aarch64 ;;
  Linux-x86_64) asset=herdr-standup-linux-x86_64 ;;
  *) echo "Unsupported platform" >&2; exit 1 ;;
esac
mkdir -p bin
base="https://github.com/neospeed83/herdr-standup/releases/download/v$version"
tmp="bin/.herdr-standup.$$"
checksum="$tmp.sha256"
trap 'rm -f "$tmp" "$checksum"' EXIT HUP INT TERM
curl -fsSL "$base/$asset" -o "$tmp"
curl -fsSL "$base/$asset.sha256" -o "$checksum"
expected=$(awk '{print $1}' "$checksum")
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmp" | awk '{print $1}')
else
  actual=$(shasum -a 256 "$tmp" | awk '{print $1}')
fi
[ "$actual" = "$expected" ] || { echo "Checksum verification failed" >&2; exit 1; }
chmod +x "$tmp"
mv -f "$tmp" bin/herdr-standup

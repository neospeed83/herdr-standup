#!/bin/sh
set -eu
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) asset=herdr-standup-macos-aarch64 ;;
  Darwin-x86_64) asset=herdr-standup-macos-x86_64 ;;
  Linux-aarch64|Linux-arm64) asset=herdr-standup-linux-aarch64 ;;
  Linux-x86_64) asset=herdr-standup-linux-x86_64 ;;
  *) echo "Unsupported platform" >&2; exit 1 ;;
esac
mkdir -p bin
curl -fsSL "https://github.com/neospeed83/herdr-standup/releases/latest/download/$asset" -o bin/herdr-standup
chmod +x bin/herdr-standup

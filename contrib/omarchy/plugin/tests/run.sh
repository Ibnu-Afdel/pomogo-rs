#!/usr/bin/env bash
# Headless test runner for the PomoGo bar widget.
#
# Quickshell's native plugins are linked into the quickshell binary, so a
# generic QML runtime cannot import Quickshell or Quickshell.Io. The suite
# loads the real BarWidget.qml against the stub modules in tests/stubs/ and
# drives it through an injectable control singleton.
#
# Usage: tests/run.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

runner=""
for candidate in qmltestrunner /usr/lib/qt6/bin/qmltestrunner /usr/lib/qt5/bin/qmltestrunner; do
  if command -v "$candidate" >/dev/null 2>&1; then
    runner="$(command -v "$candidate")"
    break
  fi
done

if [[ -z "$runner" ]]; then
  echo "qmltestrunner not found; install the Qt declarative package to run this suite" >&2
  exit 127
fi

# Run headless. A graphical session exports QT_QPA_PLATFORM=wayland;xcb and
# QT_QPA_PLATFORMTHEME=gtk3, and the GTK theme initializes GTK, which needs an
# X display, so on a headless machine the runner would abort before the first
# test. The suite renders nothing, so force a platform and an empty theme.
export QT_QPA_PLATFORM=offscreen
export QT_QPA_PLATFORMTHEME=

exec "$runner" -import "$here/stubs" -input "$here"

#!/bin/bash
set -euo pipefail
if [[ $(uname -s) != Darwin ]]; then
  echo 'Native calendar tests require macOS.'
  exit 0
fi
calendar_test_dir=$(mktemp -d)
trap 'rm -rf "$calendar_test_dir"' EXIT
xcrun clang -fobjc-arc -fblocks -framework Foundation -framework EventKit \
  "$(dirname "$0")/calendar_macos.m" -o "$calendar_test_dir/calendar-tests"
"$calendar_test_dir/calendar-tests"

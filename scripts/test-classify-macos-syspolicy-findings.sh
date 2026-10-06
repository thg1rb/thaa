#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/classify-macos-syspolicy-findings.sh"
test_dir="$(mktemp -d "${TMPDIR:-/tmp}/thaa-syspolicy-test.XXXXXX")"
trap 'rm -rf "$test_dir"' EXIT

write_report() {
  cat >"$test_dir/report.txt"
}

expect_result() {
  local description="$1"
  local expected_status="$2"
  local actual_status=0
  if thaa_check_syspolicy_findings "$test_dir/report.txt" "$3" >"$test_dir/output.txt" 2>&1; then
    actual_status=0
  else
    actual_status=$?
  fi
  if [[ "$actual_status" -ne "$expected_status" ]]; then
    cat "$test_dir/output.txt" >&2
    echo "FAIL: $description (expected $expected_status, got $actual_status)" >&2
    exit 1
  fi
  echo "PASS: $description"
}

write_report <<'EOF'
Adhoc Signed App
    Severity: Warning
Notary Ticket Missing
    Severity: Fatal
EOF
expect_result 'expected zero-budget findings' 0 70

write_report <<'EOF'
Internal Xprotect Error
    Severity: Fatal
EOF
ImageOS=macos15 ImageVersion=20260907.0337.1
export ImageOS ImageVersion
expect_result 'control-reproduced XProtect diagnostic on validated macOS 15 image' 0 70
ImageOS=macos26 ImageVersion=20260907.0351.1
export ImageOS ImageVersion
expect_result 'control-reproduced XProtect diagnostic on validated macOS 26 image' 0 70
ImageOS=macos27 ImageVersion=unknown
export ImageOS ImageVersion
expect_result 'XProtect diagnostic on unvalidated image fails closed' 1 70

write_report <<'EOF'
Codesign Error
    Severity: Fatal
EOF
expect_result 'codesign finding fails closed' 1 70

write_report <<'EOF'
Future Security Finding
    Severity: Fatal
EOF
expect_result 'unknown finding fails closed' 1 70

: >"$test_dir/report.txt"
expect_result 'unrecognized nonzero diagnostic fails closed' 1 70
expect_result 'empty successful diagnostic passes' 0 0
printf '%s\n' 'Internal Xprotect Error occurred unexpectedly' >"$test_dir/report.txt"
expect_result 'unstructured output fails closed even with success exit' 1 0
printf '\n  \n' >"$test_dir/report.txt"
expect_result 'whitespace-only successful output passes' 0 0

echo 'All macOS syspolicy finding classification tests passed.'

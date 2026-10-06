#!/usr/bin/env bash
set -euo pipefail

app_path="${1:?usage: verify-macos-release-app.sh APP_PATH VERSION [ARCH]}"
expected_version="${2:?usage: verify-macos-release-app.sh APP_PATH VERSION [ARCH]}"
expected_arch="${3:-arm64}"

if [[ ! -d "$app_path" ]]; then
  echo "Application bundle does not exist: $app_path" >&2
  exit 1
fi

plist="$app_path/Contents/Info.plist"
code_resources="$app_path/Contents/_CodeSignature/CodeResources"
if [[ ! -f "$plist" ]]; then
  echo "Application bundle is missing Contents/Info.plist." >&2
  exit 1
fi
if [[ ! -f "$code_resources" ]]; then
  echo "Application bundle is missing its sealed CodeResources file." >&2
  exit 1
fi

actual_version="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$plist")"
if [[ "$actual_version" != "$expected_version" ]]; then
  echo "Expected app version $expected_version, found $actual_version." >&2
  exit 1
fi

icon_name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' "$plist")"
if [[ ! -f "$app_path/Contents/Resources/$icon_name" ]]; then
  echo "Application icon resource is missing: $icon_name" >&2
  exit 1
fi

executable_name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$plist")"
executable="$app_path/Contents/MacOS/$executable_name"
if [[ ! -f "$executable" ]]; then
  echo "Application executable is missing: $executable_name" >&2
  exit 1
fi
lipo "$executable" -verify_arch "$expected_arch"

# This is the regression gate for the v0.1.0 incident: checking executable
# metadata alone does not prove that the enclosing app bundle is sealed.
codesign --verify --deep --strict --verbose=4 "$app_path"

signature="$(codesign -dv --verbose=4 "$app_path" 2>&1)"
printf '%s\n' "$signature"
grep -Fq 'Signature=adhoc' <<< "$signature"
grep -Fq 'TeamIdentifier=not set' <<< "$signature"
if grep -Fq 'Info.plist=not bound' <<< "$signature"; then
  echo 'The application signature does not cover Info.plist.' >&2
  exit 1
fi

if command -v syspolicy_check >/dev/null 2>&1; then
  policy_output="$(mktemp)"
  if syspolicy_check distribution "$app_path" >"$policy_output" 2>&1; then
    policy_status=0
  else
    policy_status=$?
  fi
  cat "$policy_output"
  if grep -Eiq 'Code has no resources but signature indicates they must be present|code signature does not fully cover the bundle.s Info.plist' "$policy_output"; then
    rm -f "$policy_output"
    echo 'System policy diagnostics found an invalid bundle signature.' >&2
    exit 1
  fi
  rm -f "$policy_output"
  echo "syspolicy_check exit status: $policy_status (ad-hoc/notarization findings are diagnostic)."
else
  echo 'syspolicy_check is unavailable; strict codesign verification remains required.'
fi

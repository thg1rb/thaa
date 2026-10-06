#!/usr/bin/env bash
set -euo pipefail

candidate_dmg="${1:?usage: triage-macos-xprotect.sh CANDIDATE_DMG OUTPUT_DIR}"
output_dir="${2:?usage: triage-macos-xprotect.sh CANDIDATE_DMG OUTPUT_DIR}"
candidate_sha="552253c0f8706f8e92bc12e73b3d62cf361f71072ded7bced3b0f45b432a8cb4"
historical_sha="4365f2a0d91ef5865f3aaad953fdce004c80ed8425272af9ad2c5af8fb438297"
historical_url="https://github.com/thg1rb/thaa/releases/download/v0.1.0/Thaa_0.1.0_aarch64.dmg"

mkdir -p "$output_dir"
work_dir="$(mktemp -d "${RUNNER_TEMP:-/tmp}/thaa-xprotect.XXXXXX")"
candidate_device=""
historical_device=""
cleanup() {
  [[ -z "$candidate_device" ]] || hdiutil detach "$candidate_device" >/dev/null 2>&1 || true
  [[ -z "$historical_device" ]] || hdiutil detach "$historical_device" >/dev/null 2>&1 || true
  rm -rf "$work_dir"
}
trap cleanup EXIT

if [[ "$(shasum -a 256 "$candidate_dmg" | awk '{print $1}')" != "$candidate_sha" ]]; then
  echo 'The candidate DMG hash does not match the frozen artifact.' >&2
  exit 1
fi

curl --fail --location --silent --show-error --retry 2 "$historical_url" \
  --output "$work_dir/Thaa_0.1.0_aarch64.dmg"
if [[ "$(shasum -a 256 "$work_dir/Thaa_0.1.0_aarch64.dmg" | awk '{print $1}')" != "$historical_sha" ]]; then
  echo 'The historical v0.1.0 DMG hash does not match the published artifact.' >&2
  exit 1
fi

{
  echo '## Runner'
  sw_vers
  uname -a
  echo "RUNNER_OS=${RUNNER_OS:-unknown} RUNNER_ARCH=${RUNNER_ARCH:-unknown} ImageOS=${ImageOS:-unknown} ImageVersion=${ImageVersion:-unknown}"
  echo "Xcode path: $(xcode-select -p 2>&1)"
  xcodebuild -version 2>&1 || true
  clang --version | head -2
  echo '## Security data history (available entries only)'
  softwareupdate --history 2>&1 | rg -i -C 1 'XProtect|Security Data|Gatekeeper' || true
  echo '## Candidate and historical hashes'
  shasum -a 256 "$candidate_dmg" "$work_dir/Thaa_0.1.0_aarch64.dmg"
} >"$output_dir/environment.txt"

mkdir -p "$work_dir/candidate" "$work_dir/historical" "$work_dir/control"
hdiutil attach -readonly -nobrowse -mountpoint "$work_dir/candidate" "$candidate_dmg" \
  >"$output_dir/candidate-mount.txt" 2>&1
candidate_device="$(awk '$1 ~ /^\/dev\/disk[0-9]+$/ {print $1; exit}' "$output_dir/candidate-mount.txt")"
hdiutil attach -readonly -nobrowse -mountpoint "$work_dir/historical" "$work_dir/Thaa_0.1.0_aarch64.dmg" \
  >"$output_dir/historical-mount.txt" 2>&1
historical_device="$(awk '$1 ~ /^\/dev\/disk[0-9]+$/ {print $1; exit}' "$output_dir/historical-mount.txt")"

candidate_app="$work_dir/candidate/Thaa.app"
historical_app="$work_dir/historical/Thaa.app"
if [[ ! -d "$candidate_app" || ! -d "$historical_app" ]]; then
  echo 'Expected Thaa.app was not found in a mounted image.' >&2
  exit 1
fi

{
  echo '## Quarantine attributes (read only)'
  xattr -p com.apple.quarantine "$candidate_dmg" 2>&1 || true
  xattr -p com.apple.quarantine "$candidate_app" 2>&1 || true
  xattr -p com.apple.quarantine "$work_dir/Thaa_0.1.0_aarch64.dmg" 2>&1 || true
  xattr -p com.apple.quarantine "$historical_app" 2>&1 || true
  echo '## Candidate bundle file inventory'
  find "$candidate_app" -type f -print | sort
  echo '## Candidate file types'
  find "$candidate_app" -type f -exec file {} \; | sort
  echo '## Candidate file hashes'
  find "$candidate_app" -type f -exec shasum -a 256 {} \; | sort
  echo '## Candidate Mach-O code signatures'
  while IFS= read -r candidate_file; do
    if file "$candidate_file" | grep -Fq 'Mach-O'; then
      echo "$candidate_file"
      codesign --verify --strict --verbose=4 "$candidate_file"
      codesign -dv --verbose=4 "$candidate_file" 2>&1
    fi
  done < <(find "$candidate_app" -type f -print)
  echo '## Candidate version and bundle identity'
  /usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$candidate_app/Contents/Info.plist"
  /usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$candidate_app/Contents/Info.plist"
  /usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' "$candidate_app/Contents/Info.plist"
  echo '## Candidate strict bundle verification'
  codesign --verify --deep --strict --verbose=4 "$candidate_app"
  echo '## Candidate signing metadata'
  codesign -dv --verbose=4 "$candidate_app" 2>&1
  echo '## Historical strict bundle verification (failure expected)'
  historical_status=0
  codesign --verify --deep --strict --verbose=4 "$historical_app" \
    >"$output_dir/historical-codesign.txt" 2>&1 || historical_status=$?
  cat "$output_dir/historical-codesign.txt"
  echo "Historical strict verification exit: $historical_status (failure is the recorded v0.1.0 defect)."
} >"$output_dir/bundle-inventory.txt" 2>&1

mkdir -p "$work_dir/control/Control.app/Contents/MacOS"
cat >"$work_dir/control/main.c" <<'EOF'
int main(void) { return 0; }
EOF
cat >"$work_dir/control/Control.app/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>Control</string>
<key>CFBundleIdentifier</key><string>org.example.thaa-xprotect-control</string>
<key>CFBundleName</key><string>Control</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
</dict></plist>
EOF
clang -arch arm64 -o "$work_dir/control/Control.app/Contents/MacOS/Control" "$work_dir/control/main.c"
codesign --force --sign - --timestamp=none "$work_dir/control/Control.app"
codesign --verify --deep --strict --verbose=4 "$work_dir/control/Control.app" \
  >"$output_dir/control-codesign.txt" 2>&1

run_policy() {
  local label="$1"
  local app_path="$2"
  local count="$3"
  local run=1
  while (( run <= count )); do
    local result="$output_dir/${label}-${run}.txt"
    {
      printf 'START_UTC='
      date -u '+%Y-%m-%dT%H:%M:%SZ'
      printf 'COMMAND=syspolicy_check distribution %s\n' "$app_path"
      local policy_status=0
      syspolicy_check distribution "$app_path" >"$result.output" 2>&1 || policy_status=$?
      cat "$result.output"
      echo "EXIT=$policy_status"
      printf 'END_UTC='
      date -u '+%Y-%m-%dT%H:%M:%SZ'
    } >"$result" 2>&1
    run=$((run + 1))
  done
}

run_policy candidate "$candidate_app" 3
run_policy historical "$historical_app" 1
run_policy control "$work_dir/control/Control.app" 3

# This is a bounded, read-only query; unified logging redacts private fields by default.
log show --last 10m --style compact --predicate \
  '((process == "syspolicyd" OR process == "XProtectService" OR process == "amfid" OR process == "taskgated-helper" OR process == "syspolicy_check") AND (eventMessage CONTAINS[c] "xprotect" OR eventMessage CONTAINS[c] "scan" OR eventMessage CONTAINS[c] "assessment" OR eventMessage CONTAINS[c] "error" OR eventMessage CONTAINS[c] "match" OR eventMessage CONTAINS[c] "denied"))' \
  >"$output_dir/trusted-execution-logs.txt" 2>&1 || true

echo 'Triage complete. App executables and control code were not launched.'

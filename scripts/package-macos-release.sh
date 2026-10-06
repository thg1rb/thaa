#!/usr/bin/env bash
set -euo pipefail

version="${1:?usage: package-macos-release.sh VERSION}"
arch="${2:-aarch64}"
if [[ ! "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo "Invalid release version: $version (expected MAJOR.MINOR.PATCH)" >&2
  exit 1
fi
target_arch="arm64"
if [[ "$arch" != "aarch64" ]]; then
  echo "Unsupported macOS release artifact architecture: $arch" >&2
  exit 1
fi

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bundle_dir="$root_dir/src-tauri/target/release/bundle"
app="$bundle_dir/macos/Thaa.app"
dmg_dir="$bundle_dir/dmg"
dmg="$dmg_dir/Thaa_${version}_${arch}.dmg"

compare_signed_bundle_files() {
  local reference_app="$1"
  local comparison_app="$2"
  local executable_name
  executable_name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$reference_app/Contents/Info.plist")"
  local relative_path
  for relative_path in \
    "Contents/Info.plist" \
    "Contents/MacOS/$executable_name" \
    "Contents/Resources/icon.icns" \
    "Contents/_CodeSignature/CodeResources"; do
    local reference_hash
    local comparison_hash
    reference_hash="$(shasum -a 256 "$reference_app/$relative_path" | awk '{print $1}')"
    comparison_hash="$(shasum -a 256 "$comparison_app/$relative_path" | awk '{print $1}')"
    if [[ "$reference_hash" != "$comparison_hash" ]]; then
      echo "DMG packaging changed signed bundle content: $relative_path" >&2
      exit 1
    fi
  done
}

cd "$root_dir"
pnpm tauri build --bundles app
"$root_dir/scripts/verify-macos-release-app.sh" "$app" "$version" "$target_arch"

# Package the exact verified bundle; do not invoke a second bundling pass that
# could assemble or mutate a different app after the pre-DMG signature gate.
stage_dir="$(mktemp -d "${TMPDIR:-/tmp}/thaa-dmg-stage.XXXXXX")"
mount_dir="$(mktemp -d "${TMPDIR:-/tmp}/thaa-dmg-mount.XXXXXX")"
cleanup() {
  diskutil eject "$mount_dir" >/dev/null 2>&1 || true
  rm -rf "$stage_dir" "$mount_dir"
}
trap cleanup EXIT

ditto "$app" "$stage_dir/Thaa.app"
ln -s /Applications "$stage_dir/Applications"
"$root_dir/scripts/verify-macos-release-app.sh" "$stage_dir/Thaa.app" "$version" "$target_arch"
compare_signed_bundle_files "$app" "$stage_dir/Thaa.app"

mkdir -p "$dmg_dir"
rm -f "$dmg"
hdiutil create -volname Thaa -srcfolder "$stage_dir" -ov -format UDZO "$dmg"
hdiutil verify "$dmg"
diskutil image attach --readOnly --mountOptions nobrowse --mountPoint "$mount_dir" "$dmg"
mounted_app="$mount_dir/Thaa.app"
"$root_dir/scripts/verify-macos-release-app.sh" "$mounted_app" "$version" "$target_arch"
compare_signed_bundle_files "$app" "$mounted_app"

cd "$dmg_dir"
shasum -a 256 "$(basename "$dmg")" > SHA256SUMS.txt
shasum -a 256 --check SHA256SUMS.txt

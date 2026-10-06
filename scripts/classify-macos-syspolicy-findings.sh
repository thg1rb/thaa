#!/usr/bin/env bash

# Classify syspolicy_check's structured finding headings. Strict codesign
# verification runs before this helper and remains the bundle-integrity gate.
thaa_check_syspolicy_findings() {
  local report_file="${1:?report file required}"
  local exit_status="${2:?syspolicy_check exit status required}"
  local findings
  local finding
  local classification_status=0
  local unstructured_output

  findings="$(awk '
    /^[[:space:]]*$/ { next }
    /^App has failed one or more pre-distribution checks\.$/ { next }
    /^-{10,}$/ { next }
    /^[[:alnum:]][[:alnum:] .-]*$/ {
      if (finding && !has_severity) print "__UNSTRUCTURED_SYSPOLICY_OUTPUT__"
      print $0
      finding = 1
      has_severity = 0
      previous_field = ""
      next
    }
    /^    (File|Severity|Full Error|Type):/ {
      if (!finding) {
        print "__UNSTRUCTURED_SYSPOLICY_OUTPUT__"
        next
      }
      if ($0 ~ /^    Severity:/) {
        if (has_severity) print "__UNSTRUCTURED_SYSPOLICY_OUTPUT__"
        has_severity = 1
      }
      previous_field = $0
      next
    }
    /^[[:space:]]+/ && previous_field ~ /^[[:space:]]+Full Error:/ {
      continuation = $0
      match(continuation, /[^[:space:]]/)
      indent = RSTART - 1
      sub(/^[[:space:]]+/, "", continuation)
      if (indent >= 8 && continuation !~ /^[[:alnum:] .-]+:/) next
      print "__UNSTRUCTURED_SYSPOLICY_OUTPUT__"
      next
    }
    { print "__UNSTRUCTURED_SYSPOLICY_OUTPUT__" }
    END {
      if (finding && !has_severity) print "__UNSTRUCTURED_SYSPOLICY_OUTPUT__"
    }
  ' "$report_file")"

  unstructured_output="$(grep -F '__UNSTRUCTURED_SYSPOLICY_OUTPUT__' <<< "$findings" || true)"
  if [[ -n "$unstructured_output" ]]; then
    echo "::error::syspolicy_check produced unrecognized report content (exit $exit_status); review required."
    return 1
  fi

  if [[ -z "$findings" ]]; then
    if grep -q '[^[:space:]]' "$report_file"; then
      echo "::error::syspolicy_check produced unrecognized output with no structured finding (exit $exit_status); review required."
      return 1
    fi
    if [[ "$exit_status" -ne 0 ]]; then
      echo "::error::syspolicy_check exited $exit_status without a recognized finding; review required."
      return 1
    fi
    echo 'syspolicy_check exited 0 with no findings.'
    return 0
  fi

  while IFS= read -r finding; do
    case "$finding" in
      'Adhoc Signed App'|'Notary Ticket Missing')
        echo "Expected zero-budget distribution finding: $finding"
        ;;
      'Internal Xprotect Error')
        case "${ImageOS:-}:${ImageVersion:-}" in
          'macos15:20260907.0337.1'|'macos26:20260907.0351.1')
            echo "::warning::Known GitHub-hosted XProtect diagnostic on ${ImageOS}/${ImageVersion}; reproduced with a minimal ad-hoc control in R001.1 run 37409005288."
            ;;
          *)
            echo "::error::Internal Xprotect Error on an unvalidated environment (${ImageOS:-unknown}/${ImageVersion:-unknown}); release review required."
            classification_status=1
            ;;
        esac
        ;;
      '__UNSTRUCTURED_SYSPOLICY_OUTPUT__')
        # Handled before classification; keep this defensive in case the
        # structured report format changes during processing.
        classification_status=1
        ;;
      *)
        echo "::error::Unclassified syspolicy_check finding: $finding"
        classification_status=1
        ;;
    esac
  done <<< "$findings"

  if [[ "$exit_status" -ne 0 && "$classification_status" -eq 0 ]]; then
    echo "syspolicy_check exited $exit_status with only classified findings."
  elif [[ "$exit_status" -ne 0 ]]; then
    echo "syspolicy_check exited $exit_status with an unclassified/security finding."
  fi

  return "$classification_status"
}

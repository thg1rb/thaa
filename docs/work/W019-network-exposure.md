# W019 — Network Exposure

Status: READY FOR REVIEW
Requirement: binding-explanation portion of FR-016; PR-009
Priority: P2

## Objective and scope

Explain the bind scope of each observed listener using its existing normalized
local address. This work does not test reachability. It does not implement
port-conflict assistance, active network probing, firewall/NAT inspection,
interface enumeration, container discovery, or W020 runtime detection.

The address remains attached to its listener endpoint, even when several
listeners belong to one process. The UI classifies loopback, IPv4 wildcard,
IPv6 wildcard, specific address, or unknown. A specific non-loopback address is
shown literally and is not further labeled private, LAN, or public.

## Decisions

- Reuse the `IpAddr` already collected by the macOS and Windows listener
  providers. Classification is deterministic shared-domain work; no new scan,
  subprocess, DNS lookup, interface enumeration, or external service is used.
- `0.0.0.0` is IPv4 wildcard; `::` is IPv6 wildcard. The IPv6 label does not
  imply IPv4 dual-stack behavior.
- IPv4-mapped IPv6 loopback addresses classify as loopback. Other mapped
  addresses remain specific IPv6 addresses; the original address remains the
  displayed endpoint value.
- Known non-loopback, non-unspecified addresses are `SpecificAddress`. The
  exact address is visible; address class does not establish reachability.
- Multicast and IPv4 limited-broadcast addresses are `Unknown`; they are not
  presented as ordinary interface-specific unicast binds.
- Scoped IPv6 addresses with a nonzero scope ID remain unknown because the
  current domain `IpAddr` cannot retain that identifier.
- Exposure text is factual and neutral. Wildcard/specific binding does not
  establish firewall permission, LAN/Internet reachability, NAT forwarding,
  or VM/container forwarding. No process action, search, or process identity
  behavior depends on exposure.

## Security and performance

The main W019 risk is misleading interpretation. Classification must state only
what the local socket address shows. Native address parsing remains in the
existing adapters and the classifier operates on typed IP addresses. Work is
constant-time per listener and does not add network or system enumeration.

## Acceptance criteria

| ID    | Criterion                                                                                                       | Evidence                                    | Status  |
| ----- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------- | ------- |
| AC-01 | Each listener receives deterministic bind-scope classification from its observed address.                       | Domain and DTO tests                        | PASS    |
| AC-02 | IPv4/IPv6 loopback, wildcard, specific, mapped IPv4, and unavailable address cases are handled correctly.       | Domain table tests                          | PASS    |
| AC-03 | Wildcard and specific labels do not claim Internet/LAN reachability or dual-stack behavior.                     | UI copy/accessibility tests                 | PASS    |
| AC-04 | Mixed listeners owned by one process retain independent exposure values.                                        | Forest regression test                      | PASS    |
| AC-05 | Search, Refresh, Process Tree, metrics, and process actions retain their existing behavior.                     | Existing and updated frontend/runtime tests | PASS    |
| AC-06 | macOS and Windows continue using existing structured local-address data without elevation or added enumeration. | Existing provider tests; native CI pending  | PENDING |
| AC-07 | Documentation records the bind-scope/reachability boundary and current scoped-IPv6 limitation.                  | Documentation checks                        | PASS    |
| AC-08 | No release/version/tag changes or W020 work is included.                                                        | Final diff review pending                   | PENDING |

## Validation plan

Run domain/DTO and UI tests, all relevant Rust and frontend checks, docs
format/link validation, dependency/security audits, `git diff --check`, and
macOS/Windows Native Validation in CI. Test safe local listener examples on
macOS if available. Report interactive Windows validation separately.

## Progress

- Discovery confirms FR-016's factual common-binding explanation is the W019
  requirement slice; port-conflict assistance remains deferred.
- Existing provider address data is sufficient. No provider enumeration or
  dependency change is planned.
- Implemented shared classification, DTO variants, per-listener labels/help,
  mixed-listener coverage, and mapped IPv4 loopback handling. Mapped non-loopback
  addresses stay specific. Scoped IPv6 remains explicitly unknown under the
  existing `IpAddr` model.
- Rust tests/Clippy, frontend formatting/lint/typecheck/tests/build, docs
  formatting/links, audits, `git diff --check`, and the macOS native no-bundle
  build pass. Required PR CI and review remain pending.
- Interactive macOS UI validation was not run: the desktop session contained
  a separately launched Thaa instance showing active user processes, so it was
  not replaced or used as evidence for this source tree. Interactive Windows
  validation is unavailable locally and remains NOT RUN.

import type { RuntimeEntry } from "../model/types";

export function safeDisplay(value: string) {
  return Array.from(value, (character) => {
    const point = character.codePointAt(0) ?? 0;
    return point < 0x20 ||
      (point >= 0x7f && point <= 0x9f) ||
      (point >= 0x202a && point <= 0x202e) ||
      (point >= 0x2066 && point <= 0x2069)
      ? "�"
      : character;
  }).join("");
}

export function availableText(value: { state: string; value?: string }) {
  return value.state === "available" && value.value
    ? safeDisplay(value.value)
    : null;
}

export function displayName(entry: RuntimeEntry) {
  if (entry.process.state === "available")
    return availableText(entry.process.details.name) ?? "Unknown process";
  return entry.process.state === "unavailable"
    ? "Process details unavailable"
    : "Unknown process";
}

export function bindingLabel(binding: RuntimeEntry["binding"]) {
  if (binding === "loopbackOnly") return "Loopback only";
  if (binding === "wildcardIpv4") return "All IPv4 interfaces";
  if (binding === "wildcardIpv6") return "All IPv6 interfaces";
  if (binding === "specificAddress") return "Specific address";
  return "Address unknown";
}

export function bindingDescription(
  binding: RuntimeEntry["binding"],
  localAddress: string | null,
) {
  if (binding === "loopbackOnly") {
    return "Bound to a loopback address for host-local connections.";
  }
  if (binding === "wildcardIpv4") {
    return "Bound to the IPv4 wildcard address. Eligible local IPv4 interfaces may accept connections; this does not prove LAN or Internet reachability.";
  }
  if (binding === "wildcardIpv6") {
    return "Bound to the IPv6 wildcard address. Eligible local IPv6 interfaces may accept connections; IPv4 dual-stack behavior and remote reachability are not inferred.";
  }
  if (binding === "specificAddress") {
    return localAddress
      ? `Bound to the specific local address ${safeDisplay(localAddress)}. This does not establish whether another device can reach it.`
      : "Bound to a specific address, but its display value is unavailable. Remote reachability is not established.";
  }
  return "The local bind address is unavailable, so its scope cannot be determined.";
}

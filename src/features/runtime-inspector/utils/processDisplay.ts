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
  if (binding === "potentiallyReachable") return "Beyond loopback";
  return "Address unknown";
}

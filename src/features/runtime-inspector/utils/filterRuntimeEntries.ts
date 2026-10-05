import type { RuntimeEntry } from "../model/types";
import { displayName } from "./processDisplay";

export function normalizeRuntimeQuery(query: string) {
  return query.trim().replace(/\s+/gu, " ").toLowerCase();
}

function numericPortQuery(query: string) {
  if (!/^\d+$/u.test(query)) return null;

  const port = Number(query);
  return Number.isSafeInteger(port) && port >= 0 && port <= 65_535
    ? port
    : null;
}

export function filterRuntimeEntries(
  entries: RuntimeEntry[],
  query: string,
): RuntimeEntry[] {
  const normalizedQuery = normalizeRuntimeQuery(query);
  if (normalizedQuery.length === 0) return entries;

  const portQuery = numericPortQuery(normalizedQuery);
  return entries.filter((entry) => {
    const processName = normalizeRuntimeQuery(displayName(entry));
    return (
      processName.includes(normalizedQuery) ||
      (portQuery !== null && entry.port === portQuery)
    );
  });
}

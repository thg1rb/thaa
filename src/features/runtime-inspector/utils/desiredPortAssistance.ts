import type { RuntimeEntry } from "../model/types";
import { normalizeRuntimeQuery } from "./filterRuntimeEntries";

export type DesiredPortAssistance = {
  port: number;
  matchingEntries: RuntimeEntry[];
};

/**
 * Derives an exact desired-port query from the current listener snapshot.
 * Search has broader OR semantics, so this intentionally matches only ports.
 */
export function deriveDesiredPortAssistance(
  entries: RuntimeEntry[],
  query: string,
): DesiredPortAssistance | null {
  const normalizedQuery = normalizeRuntimeQuery(query);
  if (!/^\d+$/u.test(normalizedQuery)) return null;

  const port = Number(normalizedQuery);
  if (!Number.isSafeInteger(port) || port < 1 || port > 65_535) return null;

  return {
    port,
    matchingEntries: entries.filter((entry) => entry.port === port),
  };
}

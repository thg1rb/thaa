import type { RuntimeEntry } from "../model/types";
import { displayName } from "./processDisplay";

export type TrayProcessGroup = {
  key: string;
  identityKnown: boolean;
  processId: number | null;
  name: string;
  runtime: RuntimeEntry["runtime"];
  entries: RuntimeEntry[];
};

/** Group only entries whose PID and observed start time identify one instance. */
export function groupTrayEntries(entries: RuntimeEntry[]): TrayProcessGroup[] {
  const groups = new Map<string, TrayProcessGroup>();

  for (const entry of entries) {
    const process =
      entry.process.state === "available" ? entry.process.details : null;
    const startTime =
      process?.startTimeUnixMs.state === "available"
        ? process.startTimeUnixMs.value
        : null;
    const identityKnown =
      entry.processId !== null &&
      process?.processId === entry.processId &&
      startTime !== null;
    const key = identityKnown
      ? `process:${entry.processId}:${startTime}`
      : `entry:${entry.entryRef}`;
    const current = groups.get(key);

    if (current) {
      current.entries.push(entry);
      if (current.runtime === null) current.runtime = entry.runtime;
      continue;
    }

    groups.set(key, {
      key,
      identityKnown,
      processId: identityKnown ? entry.processId : null,
      name: displayName(entry),
      runtime: entry.runtime,
      entries: [entry],
    });
  }

  return [...groups.values()]
    .map((group) => ({
      ...group,
      entries: [...group.entries].sort(
        (left, right) =>
          left.port - right.port ||
          compareText(left.localAddress ?? "", right.localAddress ?? ""),
      ),
    }))
    .sort(
      (left, right) =>
        compareText(left.name.toLowerCase(), right.name.toLowerCase()) ||
        (left.processId ?? Number.MAX_SAFE_INTEGER) -
          (right.processId ?? Number.MAX_SAFE_INTEGER) ||
        compareText(left.key, right.key),
    );
}

function compareText(left: string, right: string) {
  return left < right ? -1 : left > right ? 1 : 0;
}

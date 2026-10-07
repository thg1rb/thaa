import type { RuntimeEntry } from "../model/types";
import { displayName } from "./processDisplay";
import { normalizeRuntimeQuery } from "./filterRuntimeEntries";

export type ProcessNode = {
  key: string;
  processId: number | null;
  name: string;
  entries: RuntimeEntry[];
  children: ProcessNode[];
  contextOnly: boolean;
};

/** Derive a snapshot-scoped forest; ambiguous or cyclic PID edges are dropped. */
export function buildProcessForest(
  entries: RuntimeEntry[],
  query: string,
): ProcessNode[] {
  const normalized = normalizeRuntimeQuery(query);
  const groups = new Map<string, ProcessNode>();
  const keysByPid = new Map<number, string[]>();
  const startByKey = new Map<string, number | null>();
  const parentsByKey = new Map<string, Set<number | null>>();
  const matched = new Set<string>();
  const matchedEntries = new Set<string>();

  for (const entry of entries) {
    const pid = entry.processId;
    const start =
      entry.process.state === "available"
        ? entry.process.details.startTimeUnixMs.state === "available"
          ? entry.process.details.startTimeUnixMs.value
          : null
        : null;
    const key =
      pid === null ? `entry:${entry.entryRef}` : `${pid}:${start ?? "unknown"}`;
    let node = groups.get(key);
    if (!node) {
      node = {
        key,
        processId: pid,
        name: displayName(entry),
        entries: [],
        children: [],
        contextOnly: false,
      };
      groups.set(key, node);
      startByKey.set(key, start);
      if (pid !== null) {
        const keys = keysByPid.get(pid);
        if (keys) keys.push(key);
        else keysByPid.set(pid, [key]);
      }
    }
    node.entries.push(entry);
    const observedParents = parentsByKey.get(key) ?? new Set<number | null>();
    observedParents.add(entry.parentProcessId);
    parentsByKey.set(key, observedParents);
    const nameMatch = normalizeRuntimeQuery(displayName(entry)).includes(
      normalized,
    );
    const portMatch =
      /^\d+$/u.test(normalized) && Number(normalized) === entry.port;
    if (!normalized || nameMatch || portMatch) {
      matched.add(key);
      matchedEntries.add(entry.entryRef);
    }
  }

  const parent = new Map<string, string>();
  if (normalized) {
    // Matching rows are retained. Parent nodes are added only as context.
    for (const node of groups.values()) {
      node.entries = node.entries.filter((entry) =>
        matchedEntries.has(entry.entryRef),
      );
    }
  }
  for (const [key, node] of groups) {
    if (node.processId === null) continue;
    if (startByKey.get(key) === null) continue;
    const observedParents = parentsByKey.get(key);
    if (!observedParents || observedParents.size !== 1) continue;
    const parentPid = observedParents.values().next().value;
    if (
      parentPid === undefined ||
      parentPid === null ||
      parentPid === node.processId
    )
      continue;
    const candidates = keysByPid.get(parentPid) ?? [];
    const parentKey = candidates.length === 1 ? candidates[0] : undefined;
    const childStart = startByKey.get(key);
    const parentStart = parentKey ? startByKey.get(parentKey) : null;
    if (
      parentKey &&
      parentKey !== key &&
      parentStart !== null &&
      parentStart !== undefined &&
      childStart !== null &&
      childStart !== undefined &&
      parentStart <= childStart
    )
      parent.set(key, parentKey);
  }

  // Drop every edge participating in a cycle. The input size is bounded by
  // listener-owner processes and this iterative walk avoids recursive hazards.
  const visited = new Set<string>();
  for (const key of parent.keys()) {
    if (visited.has(key)) continue;
    const path: string[] = [];
    const position = new Map<string, number>();
    let current: string | undefined = key;
    while (current && parent.has(current) && !visited.has(current)) {
      const cycleStart = position.get(current);
      if (cycleStart !== undefined) {
        for (const cycleNode of path.slice(cycleStart))
          parent.delete(cycleNode);
        break;
      }
      position.set(current, path.length);
      path.push(current);
      current = parent.get(current);
    }
    for (const item of path) visited.add(item);
  }

  if (normalized) {
    const contextVisited = new Set<string>();
    for (const key of matched) {
      let current = parent.get(key);
      while (current && !contextVisited.has(current)) {
        const ancestor = groups.get(current);
        if (!ancestor) break;
        contextVisited.add(current);
        ancestor.contextOnly = ancestor.entries.length === 0;
        current = parent.get(current);
      }
    }
  }
  for (const [key, parentKey] of parent) {
    const child = groups.get(key);
    const parentNode = groups.get(parentKey);
    if (child && parentNode) parentNode.children.push(child);
  }
  const roots = [...groups.values()].filter(
    (node) => !parent.has(node.key) || !groups.has(parent.get(node.key)!),
  );
  const keep = new Map<string, boolean>();
  const stack = roots.map((node) => ({ node, visited: false }));
  while (stack.length > 0) {
    const item = stack.pop()!;
    if (!item.visited) {
      stack.push({ node: item.node, visited: true });
      for (const child of item.node.children)
        stack.push({ node: child, visited: false });
      continue;
    }
    item.node.children = item.node.children.filter((child) =>
      keep.get(child.key),
    );
    keep.set(
      item.node.key,
      !normalized ||
        item.node.entries.length > 0 ||
        item.node.children.length > 0,
    );
  }
  return roots.filter((node) => keep.get(node.key));
}

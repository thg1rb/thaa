import type { Action, RuntimeEntry, RuntimeSnapshot } from "../model/types";
import { useMemo, useState } from "react";
import type { CSSProperties } from "react";
import { RuntimeCard } from "./RuntimeCard";
import { buildProcessForest, type ProcessNode } from "../utils/processForest";

export function RuntimeList({
  snapshot,
  query,
  pendingTargets,
  iconSources,
  iconsLoading,
  onAction,
  onConfirmForce,
  onOpen,
  onCopy,
}: {
  snapshot: RuntimeSnapshot;
  query: string;
  pendingTargets: Set<string>;
  iconSources: Map<string, string>;
  iconsLoading: boolean;
  onAction: (targetRef: string, action: Action) => void;
  onConfirmForce: (entry: RuntimeEntry) => void;
  onOpen: (entryRef: string) => void;
  onCopy: (value: string, label: string) => void;
}) {
  const forest = useMemo(
    () => buildProcessForest(snapshot.entries, query),
    [snapshot.entries, query],
  );
  const allKeys = useMemo(() => {
    const keys: string[] = [];
    const pending = [...forest];
    while (pending.length > 0) {
      const node = pending.pop()!;
      keys.push(node.key);
      pending.push(...node.children);
    }
    return keys;
  }, [forest]);
  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const presentKeys = new Set(allKeys);
  const activeCollapsed = new Set(
    [...collapsed].filter((key) => presentKeys.has(key)),
  );
  const visibleNodes: { node: ProcessNode; depth: number }[] = [];
  const pendingNodes = forest.map((node) => ({ node, depth: 0 })).reverse();
  while (pendingNodes.length > 0) {
    const current = pendingNodes.pop()!;
    visibleNodes.push(current);
    const isCollapsed =
      activeCollapsed.has(current.node.key) && query.trim() === "";
    if (!isCollapsed) {
      for (
        let index = current.node.children.length - 1;
        index >= 0;
        index -= 1
      ) {
        const child = current.node.children[index];
        if (child) pendingNodes.push({ node: child, depth: current.depth + 1 });
      }
    }
  }

  return (
    <section className="runtime-list" aria-label="Listening TCP ports">
      {visibleNodes.map(({ node, depth }) => {
        const isCollapsed =
          activeCollapsed.has(node.key) && query.trim() === "";
        return (
          <div
            className="process-tree-node"
            key={node.key}
            style={{ "--tree-depth": Math.min(depth, 6) } as CSSProperties}
          >
            {node.children.length > 0 && (
              <button
                className="process-tree-toggle"
                aria-expanded={!isCollapsed}
                disabled={query.trim().length > 0}
                aria-label={`${isCollapsed ? "Expand" : "Collapse"} ${node.name}, ${node.children.length} child processes`}
                onClick={() =>
                  setCollapsed((current) => {
                    const next = new Set(
                      [...current].filter((key) => presentKeys.has(key)),
                    );
                    if (next.has(node.key)) next.delete(node.key);
                    else next.add(node.key);
                    return next;
                  })
                }
              >
                <span aria-hidden="true">{isCollapsed ? "▸" : "▾"}</span>
                <strong>{node.name}</strong>
                <span>
                  {node.children.length} child
                  {node.children.length === 1 ? "" : "ren"}
                </span>
                {node.contextOnly && (
                  <span className="process-tree-context">Context</span>
                )}
              </button>
            )}
            {node.children.length === 0 && node.contextOnly && (
              <p className="process-tree-context-label">
                {node.name} · parent context
              </p>
            )}
            {!isCollapsed &&
              node.entries.map((entry) => (
                <RuntimeCard
                  key={entry.entryRef}
                  entry={entry}
                  capabilities={snapshot.capabilities}
                  iconSource={
                    entry.processIconRef
                      ? iconSources.get(entry.processIconRef)
                      : undefined
                  }
                  iconLoading={iconsLoading && entry.processIconRef !== null}
                  busy={
                    entry.actionTargetRef !== null &&
                    pendingTargets.has(entry.actionTargetRef)
                  }
                  onAction={onAction}
                  onConfirmForce={() => onConfirmForce(entry)}
                  onOpen={() => onOpen(entry.entryRef)}
                  onCopy={onCopy}
                />
              ))}
          </div>
        );
      })}
    </section>
  );
}

import type { Action, RuntimeEntry, RuntimeSnapshot } from "../model/types";
import { RuntimeCard } from "./RuntimeCard";

export function RuntimeList({
  snapshot,
  pendingTargets,
  iconSources,
  iconsLoading,
  onAction,
  onConfirmForce,
  onOpen,
  onCopy,
}: {
  snapshot: RuntimeSnapshot;
  pendingTargets: Set<string>;
  iconSources: Map<string, string>;
  iconsLoading: boolean;
  onAction: (targetRef: string, action: Action) => void;
  onConfirmForce: (entry: RuntimeEntry) => void;
  onOpen: (entryRef: string) => void;
  onCopy: (value: string, label: string) => void;
}) {
  return (
    <section className="runtime-list" aria-label="Listening TCP ports">
      {snapshot.entries.map((entry) => (
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
    </section>
  );
}

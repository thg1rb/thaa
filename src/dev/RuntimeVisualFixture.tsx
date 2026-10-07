import { useState } from "react";
import { useToast } from "../shared/ui/toast/useToast";
import { ForceStopDialog } from "../features/runtime-inspector/components/ForceStopDialog";
import { RuntimeCard } from "../features/runtime-inspector/components/RuntimeCard";
import { RuntimeHeader } from "../features/runtime-inspector/components/RuntimeHeader";
import { RuntimeCardSkeleton } from "../features/runtime-inspector/components/RuntimeCardSkeleton";
import type {
  RuntimeEntry,
  RuntimeSnapshot,
} from "../features/runtime-inspector/model/types";
import "../features/runtime-inspector/runtime-inspector.css";

const iconSvg =
  "data:image/svg+xml," +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40"><rect x="5" y="5" width="30" height="30" rx="7" fill="#203c44"/><path d="M12 14h16v12H12zm5 4h6m-6 4h4" fill="none" stroke="#7fe7d6" stroke-width="2"/></svg>',
  );

const fixtureSnapshot: RuntimeSnapshot = {
  generation: 1,
  observedAtUnixMs: Date.now(),
  completeness: { state: "complete" },
  capabilities: { gracefulStop: true, forceStop: true },
  entries: [
    {
      entryRef: "fixture-entry-app",
      protocol: "tcp",
      localAddress: "127.0.0.1",
      port: 4173,
      binding: "loopbackOnly",
      processId: 43100,
      process: {
        state: "available",
        details: {
          processId: 43100,
          name: { state: "available", value: "Fixture Service" },
          executablePath: {
            state: "available",
            value: "/Applications/Fixture Service.app/Contents/MacOS/fixture",
          },
          startTimeUnixMs: { state: "available", value: Date.now() },
          commandArguments: { state: "unavailable", value: "Not shown" },
          workingDirectory: { state: "available", value: "/tmp/thaa-fixture" },
        },
      },
      projectRoot: "/Users/example/projects/thaa-fixture",
      gitContext: {
        repositoryRoot: "/Users/example/projects/thaa",
        branch: { state: "named", name: "feature/w016-git-context" },
      },
      localUrl: "http://127.0.0.1:4173",
      actionTargetRef: "fixture-target-app",
      processIconRef: "fixture-icon-app",
    },
    {
      entryRef: "fixture-entry-unknown",
      protocol: "tcp",
      localAddress: "0.0.0.0",
      port: 8080,
      binding: "potentiallyReachable",
      processId: null,
      process: { state: "noOwner" },
      projectRoot: null,
      gitContext: null,
      localUrl: null,
      actionTargetRef: null,
      processIconRef: null,
    },
  ],
  processIcons: [
    {
      reference: "fixture-icon-app",
      pngBase64: "",
    },
  ],
};

export function RuntimeVisualFixture({
  mode,
}: {
  mode: "cards" | "loading" | "icon-loading";
}) {
  const { showToast } = useToast();
  const [confirming, setConfirming] = useState<RuntimeEntry | null>(null);
  const showSuccess = () =>
    showToast({
      tone: "success",
      title: "Stop request sent",
      message: "Thaa will confirm the result on a later scan.",
    });
  const showFailure = () =>
    showToast({
      tone: "error",
      title: "Action failed",
      message: "Thaa does not have permission to perform this action.",
    });

  return (
    <main className="app-shell visual-fixture">
      <div
        className="window-drag-region"
        data-tauri-drag-region
        aria-hidden="true"
      />
      <RuntimeHeader
        snapshot={mode === "loading" ? null : fixtureSnapshot}
        visibleCount={fixtureSnapshot.entries.length}
        isSearching={false}
        loading={mode === "loading"}
        refreshing={false}
        onRefresh={() =>
          showToast({
            tone: "info",
            title: "Preview refreshed",
            message: "No process scan was made.",
          })
        }
      />
      <section
        className="fixture-controls"
        aria-label="Visual preview controls"
      >
        <span>Development-only preview · actions are mocked</span>
        <button
          className="button-secondary"
          onClick={() => {
            window.location.search = "?w0131=loading";
          }}
        >
          Preview initial loading
        </button>
        <button
          className="button-secondary"
          onClick={() => {
            window.location.search = "?w0131=icon-loading";
          }}
        >
          Preview icon loading
        </button>
        <button className="button-secondary" onClick={showSuccess}>
          Show success
        </button>
        <button className="button-danger" onClick={showFailure}>
          Show error
        </button>
      </section>
      {mode === "loading" ? (
        <>
          <p className="sr-only" role="status">
            Finding local listeners.
          </p>
          <section
            className="runtime-list runtime-list-skeleton"
            aria-label="Loading listening TCP ports"
            aria-busy="true"
          >
            {Array.from({ length: 3 }, (_, index) => (
              <RuntimeCardSkeleton key={index} />
            ))}
          </section>
        </>
      ) : (
        <section className="runtime-list" aria-label="Preview runtime entries">
          {fixtureSnapshot.entries.map((entry) => (
            <RuntimeCard
              key={entry.entryRef}
              entry={entry}
              capabilities={fixtureSnapshot.capabilities}
              iconSource={entry.processIconRef ? iconSvg : undefined}
              iconLoading={
                mode === "icon-loading" && entry.processIconRef !== null
              }
              busy={false}
              onAction={showSuccess}
              onConfirmForce={() => setConfirming(entry)}
              onOpen={showSuccess}
              onCopy={showSuccess}
            />
          ))}
        </section>
      )}
      {confirming && (
        <ForceStopDialog
          entry={confirming}
          busy={false}
          onCancel={() => setConfirming(null)}
          onConfirm={() => {
            showSuccess();
            setConfirming(null);
          }}
        />
      )}
    </main>
  );
}

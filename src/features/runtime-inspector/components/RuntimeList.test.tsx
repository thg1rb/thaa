import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { RuntimeEntry, RuntimeSnapshot } from "../model/types";
import { RuntimeList } from "./RuntimeList";

afterEach(cleanup);

function entry(
  pid: number,
  parentProcessId: number | null,
  name: string,
): RuntimeEntry {
  return {
    entryRef: `entry-${pid}`,
    protocol: "tcp",
    localAddress: "127.0.0.1",
    port: 3000 + pid,
    binding: "loopbackOnly",
    processId: pid,
    parentProcessId,
    process: {
      state: "available",
      details: {
        processId: pid,
        name: { state: "available", value: name },
        executablePath: { state: "unavailable", value: "Unavailable" },
        startTimeUnixMs: { state: "available", value: pid * 1000 },
        commandArguments: { state: "unavailable", value: "Unavailable" },
        workingDirectory: { state: "unavailable", value: "Unavailable" },
      },
    },
    resourceMetrics: null,
    projectRoot: null,
    gitContext: null,
    localUrl: null,
    actionTargetRef: null,
    processIconRef: null,
  };
}

function snapshot(
  generation: number,
  entries: RuntimeEntry[],
): RuntimeSnapshot {
  return {
    generation,
    observedAtUnixMs: generation,
    completeness: { state: "complete" },
    capabilities: { gracefulStop: false, forceStop: false },
    entries,
    processIcons: [],
  };
}

const callbacks = {
  pendingTargets: new Set<string>(),
  iconSources: new Map<string, string>(),
  iconsLoading: false,
  onAction: vi.fn(),
  onConfirmForce: vi.fn(),
  onOpen: vi.fn(),
  onCopy: vi.fn(),
};

describe("RuntimeList process-tree state", () => {
  it("forgets collapsed state when a process disappears from an accepted snapshot", () => {
    const root = entry(10, null, "server");
    const child = entry(11, 10, "worker");
    const props = { ...callbacks, query: "" };
    const view = render(
      <RuntimeList snapshot={snapshot(1, [root, child])} {...props} />,
    );

    fireEvent.click(screen.getByRole("button", { name: /Collapse server/ }));
    expect(screen.queryByText("worker")).not.toBeInTheDocument();

    view.rerender(<RuntimeList snapshot={snapshot(2, [])} {...props} />);
    view.rerender(
      <RuntimeList snapshot={snapshot(3, [root, child])} {...props} />,
    );
    expect(screen.getByText("worker")).toBeInTheDocument();
  });
});

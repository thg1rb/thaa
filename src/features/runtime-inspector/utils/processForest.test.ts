import { describe, expect, it } from "vitest";
import type { RuntimeEntry } from "../model/types";
import { buildProcessForest } from "./processForest";

function entry(
  pid: number,
  parent: number | null,
  name: string,
  port: number,
  ref = `${pid}-${port}`,
  startTime = pid * 1000,
  runtime: RuntimeEntry["runtime"] = null,
): RuntimeEntry {
  return {
    entryRef: ref,
    protocol: "tcp",
    localAddress: "127.0.0.1",
    port,
    binding: "loopbackOnly",
    processId: pid,
    parentProcessId: parent,
    process: {
      state: "available",
      details: {
        processId: pid,
        name: { state: "available", value: name },
        executablePath: { state: "unavailable", value: "Unavailable" },
        startTimeUnixMs: { state: "available", value: startTime },
        commandArguments: { state: "unavailable", value: "Unavailable" },
        workingDirectory: { state: "unavailable", value: "Unavailable" },
      },
    },
    resourceMetrics: null,
    runtime,
    projectRoot: null,
    gitContext: null,
    localUrl: null,
    actionTargetRef: `target-${pid}`,
    processIconRef: null,
  };
}

describe("buildProcessForest", () => {
  it("groups multiple listener rows under one process and builds a forest", () => {
    const mixedExposureWildcard = entry(10, null, "server", 8080);
    mixedExposureWildcard.binding = "wildcardIpv4";
    mixedExposureWildcard.localAddress = "0.0.0.0";
    const [root] = buildProcessForest(
      [
        entry(10, null, "server", 3000),
        mixedExposureWildcard,
        entry(11, 10, "worker", 4000),
        entry(20, null, "other", 5000),
      ],
      "",
    );
    expect(root?.processId).toBe(10);
    expect(root?.runtime).toBeNull();
    expect(root?.entries.map((item) => item.port)).toEqual([3000, 8080]);
    expect(root?.entries.map((item) => item.binding)).toEqual([
      "loopbackOnly",
      "wildcardIpv4",
    ]);
    expect(root?.children[0]?.processId).toBe(11);
  });

  it("keeps runtime metadata on the correct process node", () => {
    const forest = buildProcessForest(
      [
        entry(10, null, "node", 3000, "parent-a", 10, "nodeJs"),
        entry(10, null, "node", 3001, "parent-b", 10, "nodeJs"),
        entry(11, 10, "python", 4000, "child", 20, "python"),
      ],
      "",
    );
    expect(forest[0]?.runtime).toBe("nodeJs");
    expect(forest[0]?.children[0]?.runtime).toBe("python");
    expect(forest[0]?.entries.map((item) => item.runtime)).toEqual([
      "nodeJs",
      "nodeJs",
    ]);
  });

  it("shows a matching descendant with only its ancestor path as context", () => {
    const forest = buildProcessForest(
      [
        entry(10, null, "server", 3000),
        entry(11, 10, "worker", 4000),
        entry(12, 11, "leaf", 5000),
      ],
      "5000",
    );
    expect(forest).toHaveLength(1);
    expect(forest[0]?.contextOnly).toBe(true);
    expect(forest[0]?.entries).toHaveLength(0);
    expect(forest[0]?.children[0]?.contextOnly).toBe(true);
    expect(
      forest[0]?.children[0]?.children[0]?.entries.map((item) => item.port),
    ).toEqual([5000]);
  });

  it("does not label an ancestor as context when it also matches", () => {
    const forest = buildProcessForest(
      [entry(10, null, "web-app", 3000), entry(11, 10, "web-app-worker", 4000)],
      "web-app",
    );
    expect(forest[0]?.contextOnly).toBe(false);
    expect(forest[0]?.children[0]?.contextOnly).toBe(false);
  });

  it("drops self-links, cycles, and ambiguous parent PIDs safely", () => {
    const self = buildProcessForest([entry(1, 1, "self", 1000)], "");
    expect(self[0]?.children).toHaveLength(0);
    const cycle = buildProcessForest(
      [
        entry(1, 2, "one", 1001, undefined, 1000),
        entry(2, 1, "two", 1002, undefined, 1000),
      ],
      "",
    );
    expect(cycle).toHaveLength(2);
    expect(cycle.every((node) => node.children.length === 0)).toBe(true);
  });

  it("rejects a reused parent PID whose observed process started after the child", () => {
    const forest = buildProcessForest(
      [entry(1, 2, "older-child", 1100), entry(2, null, "reused-parent", 1200)],
      "",
    );
    expect(forest).toHaveLength(2);
    expect(forest.every((node) => node.children.length === 0)).toBe(true);
  });

  it("keeps different process start times with the same PID as distinct roots", () => {
    const forest = buildProcessForest(
      [
        entry(2, 1, "reused child old", 1300, "old", 1000),
        entry(2, 1, "reused child new", 1301, "new", 2000),
        entry(1, null, "parent", 1200, "parent", 500),
      ],
      "",
    );
    const reused = forest
      .flatMap((node) => [node, ...node.children])
      .filter((node) => node.processId === 2);
    expect(reused).toHaveLength(2);
    expect(reused.map((node) => node.entries[0]?.entryRef)).toEqual([
      "old",
      "new",
    ]);
  });

  it("keeps a missing parent as a root and preserves exact-port semantics", () => {
    const entries = [
      entry(2, 99, "child", 30000),
      entry(2, 99, "child", 30001),
    ];
    const roots = buildProcessForest(entries, "");
    expect(roots).toHaveLength(1);
    expect(roots[0]?.processId).toBe(2);
    expect(roots[0]?.children).toHaveLength(0);
    expect(buildProcessForest(entries, "3000")).toEqual([]);
    const exact = buildProcessForest(
      [entry(2, 99, "child", 30000), entry(2, 99, "child", 30001)],
      "30000",
    );
    expect(exact[0]?.entries.map((item) => item.port)).toEqual([30000]);
  });
});

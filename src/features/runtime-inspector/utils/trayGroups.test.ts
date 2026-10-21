import { describe, expect, it } from "vitest";
import type { RuntimeEntry } from "../model/types";
import { groupTrayEntries } from "./trayGroups";

function entry(
  entryRef: string,
  processId: number | null,
  start: number | null,
  port: number,
  address: string | null = "127.0.0.1",
  name = "node",
): RuntimeEntry {
  return {
    entryRef,
    protocol: "tcp",
    localAddress: address,
    port,
    binding: "loopbackOnly",
    processId,
    parentProcessId: null,
    process:
      processId === null
        ? { state: "noOwner" }
        : {
            state: "available",
            details: {
              processId,
              name: { state: "available", value: name },
              executablePath: { state: "unavailable", value: "Unavailable" },
              startTimeUnixMs:
                start === null
                  ? { state: "unavailable", value: "Unavailable" }
                  : { state: "available", value: start },
              commandArguments: {
                state: "unavailable",
                value: "Unavailable",
              },
              workingDirectory: {
                state: "unavailable",
                value: "Unavailable",
              },
            },
          },
    resourceMetrics: null,
    runtime: null,
    projectRoot: null,
    gitContext: null,
    localUrl: null,
    actionTargetRef: null,
    processIconRef: null,
  };
}

describe("groupTrayEntries", () => {
  it("groups only identical PID and start-time identities and preserves endpoints", () => {
    const groups = groupTrayEntries([
      entry("one", 8, 100, 3000),
      entry("two", 8, 100, 3000, "::1"),
      entry("reused-pid", 8, 200, 3000),
    ]);

    expect(groups).toHaveLength(2);
    expect(groups.map((group) => group.key)).toEqual([
      "process:8:100",
      "process:8:200",
    ]);
    expect(groups[0]?.entries.map((item) => item.entryRef)).toEqual([
      "one",
      "two",
    ]);
  });

  it("keeps entries with missing or mismatched identity separate", () => {
    const groups = groupTrayEntries([
      entry("missing-start-a", 8, null, 3000),
      entry("missing-start-b", 8, null, 3001),
      entry("no-owner", null, null, 3000),
    ]);

    expect(groups).toHaveLength(3);
    expect(groups.every((group) => !group.identityKnown)).toBe(true);
    expect(groups.every((group) => group.entries.length === 1)).toBe(true);
  });

  it("keeps a provider PID mismatch outside a process group", () => {
    const mismatched = entry("mismatch", 8, 100, 3000);
    if (mismatched.process.state === "available")
      mismatched.process.details.processId = 9;

    const groups = groupTrayEntries([mismatched]);

    expect(groups).toHaveLength(1);
    expect(groups[0]?.identityKnown).toBe(false);
    expect(groups[0]?.key).toBe("entry:mismatch");
  });

  it("sorts entries by port then address and groups by process name", () => {
    const items = [
      entry("z", 9, 900, 3000),
      entry("a", 2, 200, 5000, "127.0.0.1", "api"),
      entry("low", 9, 900, 2000),
    ];
    const groups = groupTrayEntries(items);

    expect(groups.map((group) => group.name)).toEqual(["api", "node"]);
    expect(groups[1]?.entries.map((item) => item.port)).toEqual([2000, 3000]);
  });
});

import { describe, expect, it } from "vitest";
import type { RuntimeEntry } from "../model/types";
import { deriveDesiredPortAssistance } from "./desiredPortAssistance";

function entry(name: string, port: number, entryRef: string): RuntimeEntry {
  return {
    entryRef,
    protocol: "tcp",
    localAddress: "127.0.0.1",
    port,
    binding: "loopbackOnly",
    processId: null,
    parentProcessId: null,
    process: {
      state: "available",
      details: {
        processId: 0,
        name: { state: "available", value: name },
        executablePath: { state: "unavailable", value: "Unavailable" },
        startTimeUnixMs: { state: "unavailable", value: "Unavailable" },
        commandArguments: { state: "unavailable", value: "Unavailable" },
        workingDirectory: { state: "unavailable", value: "Unavailable" },
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

describe("deriveDesiredPortAssistance", () => {
  const first = entry("web", 3000, "first");
  const second = entry("api", 3000, "second");
  const numericName = entry("3000 worker", 4000, "name-match");

  it("selects every exact endpoint port match and excludes process-name matches", () => {
    expect(
      deriveDesiredPortAssistance([first, numericName, second], " 03000 "),
    ).toEqual({ port: 3000, matchingEntries: [first, second] });
  });

  it.each(["", "3000/tcp", "0", "65536", "-1", "3e3", "３０００"])(
    "does not create assistance for unsupported query %s",
    (query) => {
      expect(deriveDesiredPortAssistance([first], query)).toBeNull();
    },
  );

  it("accepts the supported port range boundaries", () => {
    expect(deriveDesiredPortAssistance([], "1")?.port).toBe(1);
    expect(deriveDesiredPortAssistance([], "65535")?.port).toBe(65_535);
  });

  it("does not mutate the accepted snapshot entries", () => {
    const entries = [first, numericName, second];
    const original = [...entries];
    deriveDesiredPortAssistance(entries, "3000");
    expect(entries).toEqual(original);
  });
});

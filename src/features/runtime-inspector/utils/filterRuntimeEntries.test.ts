import { describe, expect, it } from "vitest";
import type { RuntimeEntry } from "../model/types";
import { filterRuntimeEntries } from "./filterRuntimeEntries";

function entry(
  name: string | null,
  port: number,
  entryRef = `${name}-${port}`,
) {
  return {
    entryRef,
    port,
    process:
      name === null
        ? { state: "noOwner" as const }
        : {
            state: "available" as const,
            details: {
              name: { state: "available" as const, value: name },
            },
          },
  } as RuntimeEntry;
}

describe("filterRuntimeEntries", () => {
  const chrome = entry("Google Chrome", 9222);
  const chromeHelper = entry("Chrome Helper", 9223);
  const code = entry("Code", 3000);
  const nameless = entry(null, 4000);
  const entries = [chrome, code, chromeHelper, nameless];

  it.each(["", "   ", "\t\n"])('shows all entries for query "%s"', (query) => {
    expect(filterRuntimeEntries(entries, query)).toEqual(entries);
  });

  it.each([
    ["Google Chrome", [chrome]],
    ["gle chr", [chrome]],
    ["CHROME", [chrome, chromeHelper]],
    ["  code  ", [code]],
    ["google   chrome", [chrome]],
  ])("matches process display names for %s", (query, expected) => {
    expect(filterRuntimeEntries(entries, query)).toEqual(expected);
  });

  it("uses substring matching without fuzzy matches", () => {
    expect(filterRuntimeEntries(entries, "chromee")).toEqual([]);
    expect(filterRuntimeEntries(entries, "node")).toEqual([]);
  });

  it("matches only an exact valid numeric port", () => {
    const longPort = entry("Service", 30000);
    const numericName = entry("65536", 1234);
    const candidates = [code, longPort];
    expect(filterRuntimeEntries(candidates, "3000")).toEqual([code]);
    expect(filterRuntimeEntries(candidates, "300")).toEqual([]);
    expect(filterRuntimeEntries(candidates, "65536")).toEqual([]);
    expect(filterRuntimeEntries(candidates, "-1")).toEqual([]);
    expect(filterRuntimeEntries([numericName], "65536")).toEqual([numericName]);
  });

  it("accepts a numeric port at the lower boundary", () => {
    const portZero = entry("Port Zero", 0);
    expect(filterRuntimeEntries([portZero], "0")).toEqual([portZero]);
  });

  it("preserves source order and does not mutate listener entries", () => {
    const source = [chrome, code, chromeHelper];
    const sourceBefore = structuredClone(source);
    expect(filterRuntimeEntries(source, "chrome")).toEqual([
      chrome,
      chromeHelper,
    ]);
    expect(source).toEqual(sourceBefore);
    expect(source.map((item) => item.entryRef)).toEqual([
      chrome.entryRef,
      code.entryRef,
      chromeHelper.entryRef,
    ]);
  });

  it("can match either a process name or an exact port", () => {
    expect(filterRuntimeEntries(entries, "3000")).toEqual([code]);
    expect(filterRuntimeEntries(entries, "chrome")).toEqual([
      chrome,
      chromeHelper,
    ]);
  });

  it("handles missing names without inventing a process identity", () => {
    expect(filterRuntimeEntries([nameless], "unknown process")).toEqual([
      nameless,
    ]);
    expect(filterRuntimeEntries([nameless], "4000")).toEqual([nameless]);
  });
});

import { describe, expect, it } from "vitest";
import {
  formatCpuPercent,
  formatMemoryBytes,
  formatUptime,
} from "./resourceMetrics";

describe("formatCpuPercent", () => {
  it("formats useful precision and unavailable values", () => {
    expect(formatCpuPercent(0)).toBe("0%");
    expect(formatCpuPercent(3.44)).toBe("3.4%");
    expect(formatCpuPercent(100)).toBe("100%");
    expect(formatCpuPercent(null)).toBe("—");
  });
});

describe("formatMemoryBytes", () => {
  it("uses binary units and handles boundaries", () => {
    expect(formatMemoryBytes(0)).toBe("0 B");
    expect(formatMemoryBytes(1023)).toBe("1023 B");
    expect(formatMemoryBytes(1024)).toBe("1.0 KiB");
    expect(formatMemoryBytes(1024 * 1024)).toBe("1.0 MiB");
    expect(formatMemoryBytes(2 ** 40)).toBe("1.0 TiB");
    expect(formatMemoryBytes(null)).toBe("—");
  });
});

describe("formatUptime", () => {
  it("formats seconds through multi-day durations", () => {
    expect(formatUptime(0)).toBe("0s");
    expect(formatUptime(42_000)).toBe("42s");
    expect(formatUptime(8 * 60_000)).toBe("8m");
    expect(formatUptime((3 * 60 + 12) * 60_000)).toBe("3h 12m");
    expect(formatUptime((2 * 24 + 4) * 60 * 60_000)).toBe("2d 4h");
    expect(formatUptime(6 * 7 * 24 * 60 * 60_000)).toBe("42d");
    expect(formatUptime(null)).toBe("—");
  });
});

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import type { RuntimeEntry } from "../model/types";
import { deriveDesiredPortAssistance } from "../utils/desiredPortAssistance";
import { DesiredPortAssistance } from "./DesiredPortAssistance";

afterEach(cleanup);

function entry(port: number, entryRef: string): RuntimeEntry {
  return {
    entryRef,
    protocol: "tcp",
    localAddress: null,
    port,
    binding: "unknown",
    processId: null,
    parentProcessId: null,
    process: { state: "noOwner" },
    resourceMetrics: null,
    runtime: null,
    projectRoot: null,
    gitContext: null,
    localUrl: null,
    actionTargetRef: null,
    processIconRef: null,
  };
}

function renderAssistance({
  entries,
  query,
  partial = false,
  stale = false,
  hasAdditionalNameMatches = false,
}: {
  entries: RuntimeEntry[];
  query: string;
  partial?: boolean;
  stale?: boolean;
  hasAdditionalNameMatches?: boolean;
}) {
  const result = deriveDesiredPortAssistance(entries, query);
  if (!result) throw new Error("Expected a valid desired-port query");
  return render(
    <DesiredPortAssistance
      result={result}
      partial={partial}
      stale={stale}
      hasAdditionalNameMatches={hasAdditionalNameMatches}
    />,
  );
}

describe("DesiredPortAssistance", () => {
  it("describes observed matches as evidence, not a guaranteed conflict", () => {
    renderAssistance({ entries: [entry(3000, "one")], query: "3000" });

    expect(
      screen.getByRole("region", {
        name: "Observed TCP listener results for port 3000",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Observed 1 TCP listener for port 3000 in the displayed snapshot.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/may be relevant to a binding conflict/i),
    ).toBeInTheDocument();
    expect(screen.queryByText(/port is available/i)).not.toBeInTheDocument();
  });

  it("does not claim availability when no matching listener was observed", () => {
    renderAssistance({ entries: [], query: "3000" });

    expect(
      screen.getByText(
        "No TCP listener for port 3000 was observed in the displayed snapshot.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText("This does not guarantee that the port is available."),
    ).toBeInTheDocument();
  });

  it("qualifies results from partial and stale snapshots", () => {
    renderAssistance({
      entries: [entry(3000, "one")],
      query: "3000",
      partial: true,
      stale: true,
    });

    expect(
      screen.getByText("The scan was partial; some listeners may be missing."),
    ).toBeInTheDocument();
    expect(
      screen.getByText("The displayed snapshot may be out of date."),
    ).toBeInTheDocument();
  });

  it("explains that zero exact matches may still show process-name results", () => {
    renderAssistance({
      entries: [],
      query: "3000",
      hasAdditionalNameMatches: true,
    });

    expect(
      screen.getByText(/additional rows may not be listeners observed/i),
    ).toBeInTheDocument();
  });

  it("clarifies mixed exact-port and process-name search matches", () => {
    renderAssistance({
      entries: [entry(3000, "exact"), entry(4000, "name")],
      query: "3000",
      hasAdditionalNameMatches: true,
    });

    expect(
      screen.getByText(
        "Observed 1 TCP listener for port 3000 in the displayed snapshot.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/additional rows may not be listeners observed/i),
    ).toBeInTheDocument();
  });

  it("does not announce every Search keystroke through a live region", () => {
    renderAssistance({ entries: [entry(3000, "one")], query: "3000" });
    expect(screen.getByRole("region")).not.toHaveAttribute("aria-live");
  });
});

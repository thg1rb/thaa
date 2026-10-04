import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RuntimeActions } from "./RuntimeActions";

afterEach(cleanup);

describe("RuntimeActions", () => {
  it("groups Stop and Force stop in an equal-width action pair", () => {
    render(
      <RuntimeActions
        targetRef="target-1"
        capabilities={{ gracefulStop: true, forceStop: true }}
        busy={false}
        onAction={vi.fn()}
        onConfirmForce={vi.fn()}
      />,
    );

    const stop = screen.getByRole("button", { name: "Stop" });
    const forceStop = screen.getByRole("button", { name: "Force stop" });
    expect(stop.parentElement).toBe(forceStop.parentElement);
    expect(stop.parentElement).toHaveClass(
      "process-actions",
      "equal-action-pair",
    );
  });

  it("does not create a two-column pair when only Force stop is supported", () => {
    render(
      <RuntimeActions
        targetRef="target-1"
        capabilities={{ gracefulStop: false, forceStop: true }}
        busy={false}
        onAction={vi.fn()}
        onConfirmForce={vi.fn()}
      />,
    );

    const forceStop = screen.getByRole("button", { name: "Force stop" });
    expect(forceStop).toBeVisible();
    expect(forceStop.parentElement).not.toHaveClass("equal-action-pair");
  });
});

import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";

const eventState = vi.hoisted(() => ({
  listener: null as ((event: { payload: unknown }) => void) | null,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(
    (_event: string, callback: (event: { payload: unknown }) => void) => {
      eventState.listener = callback;
      return Promise.resolve(() => undefined);
    },
  ),
}));

afterEach(() => {
  cleanup();
  clearMocks();
  vi.restoreAllMocks();
  eventState.listener = null;
});

const available = <T,>(value: T) => ({ state: "available" as const, value });
const snapshot = (
  entries: object[] = [],
  capabilities = { gracefulStop: true, forceStop: true },
  generation = 1,
) => ({
  generation,
  observedAtUnixMs: Date.now(),
  completeness: { state: "complete" as const },
  capabilities,
  entries,
  processIcons: [] as { reference: string; pngBase64: string }[],
});
const row = (overrides: Record<string, unknown> = {}) => ({
  entryRef: "entry-1-0",
  protocol: "tcp" as const,
  localAddress: "127.0.0.1",
  port: 5173,
  binding: "loopbackOnly" as const,
  processId: 123,
  process: {
    state: "available" as const,
    details: {
      processId: 123,
      name: available("node"),
      executablePath: available("/tmp/node"),
      startTimeUnixMs: available(Date.now()),
      commandArguments: {
        state: "unavailable" as const,
        value: "Not provided by the operating system",
      },
      workingDirectory: {
        state: "unavailable" as const,
        value: "Not provided by the operating system",
      },
    },
  },
  localUrl: "http://127.0.0.1:5173",
  actionTargetRef: "target-1-0",
  processIconRef: null as string | null,
  ...overrides,
});

describe("runtime inspector", () => {
  it("shows loading then the real listener and process information", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot" ? snapshot([row()]) : undefined,
    );
    render(<App />);
    expect(screen.getByText("Finding local listeners")).toBeInTheDocument();
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "node" })).toBeInTheDocument();
    expect(
      screen.getByText(
        (_text, element) =>
          element?.tagName === "P" &&
          element.textContent?.includes("PID 123") === true,
      ),
    ).toBeInTheDocument();
  });

  it("renders a resolved process icon from the snapshot icon request", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot"
        ? snapshot([row({ processIconRef: "icon-1-0" })])
        : command === "get_runtime_process_icons"
          ? [
              {
                reference: "icon-1-0",
                pngBase64:
                  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+nm4kAAAAASUVORK5CYII=",
              },
            ]
          : undefined,
    );
    const { container } = render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    const image = container.querySelector(".runtime-app-icon img");
    expect(image).toHaveAttribute(
      "src",
      expect.stringContaining("data:image/png;base64,"),
    );
    expect(image).toHaveAttribute("alt", "");
  });

  it("renders runtime rows before the optional icon lookup completes", async () => {
    mockIPC((command) => {
      if (command === "get_runtime_snapshot")
        return snapshot([row({ processIconRef: "icon-1-0" })]);
      if (command === "get_runtime_process_icons")
        return new Promise(() => undefined);
      return undefined;
    });
    const { container } = render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(
      container.querySelector('[data-icon-kind="process-fallback"]'),
    ).toBeInTheDocument();
  });

  it("uses the generic fallback when native icon lookup fails", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot" ? snapshot([row()]) : undefined,
    );
    const { container } = render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(
      container.querySelector('[data-icon-kind="process-fallback"]'),
    ).toBeInTheDocument();
  });

  it("falls back when an icon asset cannot be decoded by the frontend", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot"
        ? snapshot([row({ processIconRef: "icon-1-0" })])
        : command === "get_runtime_process_icons"
          ? [{ reference: "icon-1-0", pngBase64: "invalid" }]
          : undefined,
    );
    const { container } = render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    const image = container.querySelector(".runtime-app-icon img");
    expect(image).toBeInTheDocument();
    fireEvent.error(image!);
    expect(
      container.querySelector('[data-icon-kind="process-fallback"]'),
    ).toBeInTheDocument();
  });

  it("shows the unknown-owner fallback without dropping the listener", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot"
        ? snapshot([
            row({
              processId: null,
              process: { state: "noOwner" },
              localUrl: null,
              actionTargetRef: null,
            }),
          ])
        : undefined,
    );
    const { container } = render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(
      container.querySelector('[data-icon-kind="unknown-owner"]'),
    ).toBeInTheDocument();
  });

  it("does not replace a newer tray snapshot with an older command response", async () => {
    let resolveInitial:
      ((value: ReturnType<typeof snapshot>) => void) | undefined;
    mockIPC((command) =>
      command === "get_runtime_snapshot"
        ? new Promise<ReturnType<typeof snapshot>>((resolve) => {
            resolveInitial = resolve;
          })
        : undefined,
    );
    render(<App />);
    await waitFor(() => expect(eventState.listener).not.toBeNull());

    act(() => {
      eventState.listener?.({
        payload: snapshot(
          [
            row({
              entryRef: "entry-2-0",
              port: 6000,
              actionTargetRef: "target-2-0",
            }),
          ],
          { gracefulStop: true, forceStop: true },
          2,
        ),
      });
    });
    expect(screen.getByText(":6000")).toBeInTheDocument();

    await act(async () => {
      resolveInitial?.(
        snapshot([row()], { gracefulStop: true, forceStop: true }, 1),
      );
    });
    expect(screen.getByText(":6000")).toBeInTheDocument();
    expect(screen.queryByText(":5173")).not.toBeInTheDocument();
  });

  it("renders a complete empty state", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot" ? snapshot() : undefined,
    );
    render(<App />);
    expect(
      await screen.findByText("No listening TCP ports found"),
    ).toBeInTheDocument();
  });

  it("keeps partial results visible with a warning", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot"
        ? {
            ...snapshot([row()]),
            completeness: { state: "partial", reason: "permissionDenied" },
          }
        : undefined,
    );
    render(<App />);
    expect(
      await screen.findByText("Some listeners may be missing from this scan."),
    ).toBeInTheDocument();
    expect(screen.getByText(":5173")).toBeInTheDocument();
  });

  it("shows a safe provider error and retries manually", async () => {
    let scans = 0;
    mockIPC((command) => {
      if (command === "get_runtime_snapshot")
        throw new Error("sensitive native output");
      if (command === "refresh_runtime_snapshot") {
        scans += 1;
        return snapshot([row()]);
      }
      return undefined;
    });
    render(<App />);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Thaa could not inspect listening ports",
    );
    expect(
      screen.queryByText("sensitive native output"),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(scans).toBe(1);
  });

  it("exposes only capability-supported actions and confirms force stop", async () => {
    mockIPC((command) => {
      if (command === "get_runtime_snapshot")
        return snapshot([row()], { gracefulStop: false, forceStop: true });
      if (command === "request_process_action") return { state: "requested" };
      if (command === "refresh_runtime_snapshot") return snapshot([]);
      return undefined;
    });
    render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Stop" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Force stop" }));
    expect(screen.getByRole("dialog")).toHaveTextContent(
      "Normal cleanup may not run",
    );
    expect(screen.getByRole("dialog")).toHaveTextContent("Port 5173");
    fireEvent.click(
      screen.getByRole("dialog").querySelector(".button-danger")!,
    );
    await waitFor(() =>
      expect(
        screen.getByText("No listening TCP ports found"),
      ).toBeInTheDocument(),
    );
  });

  it("shows a process failure without dropping its listener", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot"
        ? snapshot([
            row({
              process: {
                state: "unavailable",
                details: { processId: 123, reason: "Permission denied" },
              },
              actionTargetRef: null,
            }),
          ])
        : undefined,
    );
    render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(screen.getByText("Permission denied")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Force stop" }),
    ).not.toBeInTheDocument();
  });

  it("does not show graceful stop on Windows capability snapshots", async () => {
    mockIPC((command) =>
      command === "get_runtime_snapshot"
        ? snapshot([row()], { gracefulStop: false, forceStop: true })
        : undefined,
    );
    render(<App />);
    expect(await screen.findByText(":5173")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Stop" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Force stop" }),
    ).toBeInTheDocument();
  });

  it("explains an identity mismatch and refreshes without retrying the action", async () => {
    let actions = 0;
    let refreshes = 0;
    mockIPC((command) => {
      if (command === "get_runtime_snapshot") return snapshot([row()]);
      if (command === "request_process_action") {
        actions += 1;
        return {
          state: "refused",
          reason:
            "The process changed since this entry was loaded. Refresh and try again.",
        };
      }
      if (command === "refresh_runtime_snapshot") {
        refreshes += 1;
        return snapshot([row()]);
      }
      return undefined;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Stop" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "The process changed since this entry was loaded",
    );
    expect(actions).toBe(1);
    expect(refreshes).toBe(1);
    expect(screen.getByRole("button", { name: "Stop" })).toBeEnabled();
  });

  it("prevents duplicate action submissions while a request is in progress", async () => {
    let resolveAction: ((value: { state: "requested" }) => void) | undefined;
    let actions = 0;
    mockIPC((command) => {
      if (command === "get_runtime_snapshot") return snapshot([row()]);
      if (command === "request_process_action") {
        actions += 1;
        return new Promise<{ state: "requested" }>((resolve) => {
          resolveAction = resolve;
        });
      }
      if (command === "refresh_runtime_snapshot") return snapshot([row()]);
      return undefined;
    });
    render(<App />);
    const stop = await screen.findByRole("button", { name: "Stop" });
    fireEvent.click(stop);
    expect(stop).toBeDisabled();
    fireEvent.click(stop);
    expect(actions).toBe(1);
    resolveAction?.({ state: "requested" });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Stop request sent",
    );
  });

  it("runs an action observation after a scan already in progress", async () => {
    let resolveScan: ((value: ReturnType<typeof snapshot>) => void) | undefined;
    let refreshCalls = 0;
    mockIPC((command) => {
      if (command === "get_runtime_snapshot")
        return snapshot([row()], undefined, 1);
      if (command === "request_process_action") return { state: "requested" };
      if (command === "refresh_runtime_snapshot") {
        refreshCalls += 1;
        if (refreshCalls === 1) {
          return new Promise<ReturnType<typeof snapshot>>((resolve) => {
            resolveScan = resolve;
          });
        }
        return snapshot([], undefined, 3);
      }
      return undefined;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Refresh" }));
    await waitFor(() => expect(refreshCalls).toBe(1));
    fireEvent.click(screen.getByRole("button", { name: "Stop" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Stop request sent",
    );

    await act(async () => {
      resolveScan?.(snapshot([row()], undefined, 2));
    });
    await waitFor(() => expect(refreshCalls).toBe(2));
    expect(
      await screen.findByText("No listening TCP ports found"),
    ).toBeInTheDocument();
  });

  it("treats an already-exited result as stale data and observes again", async () => {
    let refreshes = 0;
    mockIPC((command) => {
      if (command === "get_runtime_snapshot") return snapshot([row()]);
      if (command === "request_process_action")
        return { state: "alreadyExited" };
      if (command === "refresh_runtime_snapshot") {
        refreshes += 1;
        return snapshot([]);
      }
      return undefined;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Stop" }));
    expect(
      await screen.findByText("No listening TCP ports found"),
    ).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Process already exited",
    );
    expect(refreshes).toBe(1);
  });

  it("shows permission denial without offering elevation or locking the row", async () => {
    mockIPC((command) => {
      if (command === "get_runtime_snapshot") return snapshot([row()]);
      if (command === "request_process_action")
        return {
          state: "failed",
          reason: "Thaa does not have permission to perform this action.",
        };
      return undefined;
    });
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Force stop" }));
    fireEvent.click(
      screen.getByRole("dialog").querySelector(".button-danger")!,
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "does not have permission",
    );
    expect(screen.getByRole("button", { name: "Force stop" })).toBeEnabled();
    expect(
      screen.queryByText(/administrator|sudo|elevate/i),
    ).not.toBeInTheDocument();
  });
});

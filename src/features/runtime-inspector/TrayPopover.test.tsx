import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it, vi } from "vitest";
import { TrayPopover } from "./TrayPopover";

const eventState = vi.hoisted(() => ({
  listeners: new Map<string, (event: { payload: unknown }) => void>(),
  hide: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(
    (name: string, callback: (event: { payload: unknown }) => void) => {
      eventState.listeners.set(name, callback);
      return Promise.resolve(() => undefined);
    },
  ),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ label: "tray-popover", hide: eventState.hide }),
}));

afterEach(() => {
  cleanup();
  clearMocks();
  vi.restoreAllMocks();
  eventState.listeners.clear();
});

const snapshot = (entries: object[] = [], generation = 3) => ({
  generation,
  observedAtUnixMs: Date.now(),
  completeness: { state: "complete" as const },
  capabilities: {
    gracefulStop: true,
    forceStop: true,
    openProjectInEditor: true,
  },
  entries,
});

const trayState = (value: ReturnType<typeof snapshot>, stale = false) => ({
  snapshot: value,
  stale,
});

function entry(entryRef: string, port: number, address = "127.0.0.1") {
  return {
    entryRef,
    protocol: "tcp",
    localAddress: address,
    port,
    binding: "loopbackOnly",
    processId: 20,
    parentProcessId: null,
    process: {
      state: "available",
      details: {
        processId: 20,
        name: { state: "available", value: "node" },
        executablePath: { state: "unavailable", value: "Unavailable" },
        startTimeUnixMs: { state: "available", value: 100 },
        commandArguments: { state: "unavailable", value: "Unavailable" },
        workingDirectory: { state: "unavailable", value: "Unavailable" },
      },
    },
    resourceMetrics: null,
    runtime: "nodeJs",
    projectRoot: null,
    gitContext: null,
    localUrl: null,
    actionTargetRef: "target-1",
    processIconRef: null,
  };
}

describe("TrayPopover", () => {
  it("shows compact port summaries and preserves distinct binding details", async () => {
    const navigate = vi.fn();
    mockIPC((command, payload) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(
          snapshot([entry("entry-a", 3000), entry("entry-b", 3000, "::1")]),
        );
      if (command === "navigate_to_runtime_entry") navigate(payload);
      return undefined;
    });

    render(<TrayPopover />);

    expect(
      await screen.findByText("2 TCP endpoints · 1 identified process"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Open node in Thaa, PID 20/ }),
    ).toBeInTheDocument();
    expect(
      screen.getAllByRole("button", { name: /Open TCP port 3000/ }),
    ).toHaveLength(1);
    fireEvent.click(
      screen.getByRole("button", { name: "Show 2 bindings for TCP port 3000" }),
    );
    expect(
      screen.getByRole("button", { name: /127\.0\.0\.1/ }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /::1/ })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /127\.0\.0\.1/ }));
    await waitFor(() =>
      expect(navigate).toHaveBeenCalledWith({
        generation: 3,
        entryRef: "entry-a",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: /::1/ }));
    await waitFor(() =>
      expect(navigate).toHaveBeenLastCalledWith({
        generation: 3,
        entryRef: "entry-b",
      }),
    );
  });

  it("uses layout-matched skeletons until the first snapshot arrives", async () => {
    let resolveSnapshot:
      ((value: ReturnType<typeof trayState>) => void) | undefined;
    const pendingSnapshot = new Promise<ReturnType<typeof trayState>>(
      (resolve) => {
        resolveSnapshot = resolve;
      },
    );
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot" ? pendingSnapshot : undefined,
    );

    const { container } = render(<TrayPopover />);
    expect(
      screen.getByRole("status", { name: "Loading runtime snapshot" }),
    ).toBeInTheDocument();
    expect(container.querySelectorAll(".tray-skeleton-row")).toHaveLength(3);
    expect(
      screen.getByRole("searchbox", {
        name: "Search listeners by process or exact port",
      }),
    ).toBeInTheDocument();

    resolveSnapshot?.(trayState(snapshot([entry("entry-a", 3000)])));
    expect(
      await screen.findByRole("button", { name: /Open TCP port 3000/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("status", { name: "Loading runtime snapshot" }),
    ).not.toBeInTheDocument();
  });

  it("keeps the accepted list visible while Refresh is in progress", async () => {
    let resolveRefresh:
      ((value: ReturnType<typeof snapshot>) => void) | undefined;
    const pendingRefresh = new Promise<ReturnType<typeof snapshot>>(
      (resolve) => {
        resolveRefresh = resolve;
      },
    );
    mockIPC((command) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(snapshot([entry("entry-a", 3000)]));
      if (command === "refresh_runtime_snapshot") return pendingRefresh;
      return undefined;
    });

    render(<TrayPopover />);
    const port = await screen.findByRole("button", {
      name: /Open TCP port 3000/,
    });
    fireEvent.click(screen.getByRole("button", { name: "Refresh runtimes" }));

    expect(port).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Refreshing…");
    expect(
      screen.getByRole("button", { name: "Refreshing runtimes" }),
    ).toBeDisabled();

    resolveRefresh?.(
      snapshot([entry("entry-a", 3000), entry("entry-b", 5173)]),
    );
    expect(
      await screen.findByRole("button", { name: /Open TCP port 5173/ }),
    ).toBeInTheDocument();
    expect(screen.queryByText("Refreshing…")).not.toBeInTheDocument();
  });

  it("offers retry through the shared Refresh action after initial failure", async () => {
    const refresh = vi.fn(() => snapshot([entry("entry-a", 3000)]));
    mockIPC((command) => {
      if (command === "get_tray_runtime_snapshot")
        throw new Error("initial provider failure");
      if (command === "refresh_runtime_snapshot") return refresh();
      return undefined;
    });

    render(<TrayPopover />);
    expect(
      await screen.findByText("No accepted runtime snapshot is available."),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Runtime data is unavailable. Try Refresh."),
    ).toBeInTheDocument();
    expect(
      screen.getAllByRole("button", { name: "Refresh runtimes" }),
    ).toHaveLength(1);

    fireEvent.click(screen.getByRole("button", { name: "Refresh runtimes" }));
    expect(
      await screen.findByRole("button", { name: /Open TCP port 3000/ }),
    ).toBeInTheDocument();
    expect(refresh).toHaveBeenCalledOnce();
  });

  it("keeps same-name process instances separate", async () => {
    const first = entry("entry-a", 3000);
    const second = entry("entry-b", 5173);
    second.processId = 21;
    second.process.details.processId = 21;
    second.process.details.startTimeUnixMs.value = 200;
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(snapshot([first, second]))
        : undefined,
    );

    render(<TrayPopover />);
    expect(
      await screen.findAllByRole("button", { name: /Open node in Thaa/ }),
    ).toHaveLength(2);
    expect(
      screen.getByRole("button", { name: /Open TCP port 3000/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Open TCP port 5173/ }),
    ).toBeInTheDocument();
  });

  it("filters the current snapshot by process name or exact port", async () => {
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(
            snapshot([
              entry("entry-a", 3000),
              { ...entry("entry-b", 83000), processId: 21 },
            ]),
          )
        : undefined,
    );
    render(<TrayPopover />);

    const search = await screen.findByRole("searchbox", {
      name: "Search listeners by process or exact port",
    });
    fireEvent.change(search, { target: { value: "3000" } });
    expect(
      screen.getByRole("button", { name: /Open TCP port 3000/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Open TCP port 83000/ }),
    ).not.toBeInTheDocument();
  });

  it("navigates with the snapshot generation and hides on Escape", async () => {
    const navigate = vi.fn();
    mockIPC((command, payload) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(snapshot([entry("entry-a", 3000)], 4));
      if (command === "navigate_to_runtime_entry") navigate(payload);
      return undefined;
    });
    render(<TrayPopover />);

    fireEvent.click(
      await screen.findByRole("button", { name: /Open TCP port 3000/ }),
    );
    await waitFor(() =>
      expect(navigate).toHaveBeenCalledWith({
        generation: 4,
        entryRef: "entry-a",
      }),
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(eventState.hide).toHaveBeenCalledOnce();
  });

  it("keeps the previous snapshot visible after a refresh failure", async () => {
    mockIPC((command) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(snapshot([entry("entry-a", 3000)]));
      if (command === "refresh_runtime_snapshot")
        throw new Error("provider failure");
      return undefined;
    });
    render(<TrayPopover />);

    fireEvent.click(
      await screen.findByRole("button", { name: "Refresh runtimes" }),
    );
    expect(
      await screen.findByText(
        "Refresh failed. Showing the last successful snapshot.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Open TCP port 3000/ }),
    ).toBeInTheDocument();
  });

  it("marks an accepted snapshot stale after a refresh failure elsewhere", async () => {
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(snapshot([entry("entry-a", 3000)]))
        : undefined,
    );
    render(<TrayPopover />);

    expect(
      await screen.findByRole("button", { name: /Open TCP port 3000/ }),
    ).toBeInTheDocument();
    eventState.listeners.get("runtime-snapshot-stale")?.({ payload: true });

    expect(
      await screen.findByText("Showing the last accepted snapshot."),
    ).toBeInTheDocument();
    expect(screen.getByText("The latest refresh failed.")).toBeInTheDocument();

    eventState.listeners.get("runtime-snapshot-updated")?.({
      payload: snapshot([entry("entry-a", 3000)], 4),
    });
    await waitFor(() =>
      expect(
        screen.queryByText("Showing the last accepted snapshot."),
      ).not.toBeInTheDocument(),
    );
  });

  it("restores stale status when an earlier refresh failed before opening", async () => {
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(snapshot([entry("entry-a", 3000)]), true)
        : undefined,
    );

    render(<TrayPopover />);

    expect(
      await screen.findByText("Showing the last accepted snapshot."),
    ).toBeInTheDocument();
    expect(screen.getByText("The latest refresh failed.")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Open TCP port 3000/ }),
    ).toBeInTheDocument();
  });
});

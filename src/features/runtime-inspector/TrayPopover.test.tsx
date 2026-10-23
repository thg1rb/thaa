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
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(
    (name: string, callback: (event: { payload: unknown }) => void) => {
      eventState.listeners.set(name, callback);
      return Promise.resolve(() => undefined);
    },
  ),
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
      screen.getByRole("button", { name: "Show bindings for TCP port 3000" }),
    );
    const bindingsPanel = screen.getByRole("region", {
      name: "TCP port 3000 bindings",
    });
    expect(bindingsPanel).toHaveTextContent("2 bindings");
    expect(bindingsPanel.querySelectorAll(".tray-binding-link")).toHaveLength(
      2,
    );
    expect(
      screen.getByRole("button", { name: /Hide bindings for TCP port 3000/ }),
    ).toHaveAttribute("aria-expanded", "true");
    expect(navigate).not.toHaveBeenCalled();
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

  it("navigates from noninteractive areas of a process card exactly once", async () => {
    const navigate = vi.fn();
    mockIPC((command, payload) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(snapshot([entry("entry-card", 3000)]));
      if (command === "navigate_to_runtime_entry") navigate(payload);
      return undefined;
    });

    const { container } = render(<TrayPopover />);
    const titleButton = await screen.findByRole("button", {
      name: /Open node in Thaa, PID 20/,
    });
    const card = container.querySelector(".tray-process");
    expect(card).not.toBeNull();

    const clickTargets = [
      titleButton.querySelector("strong"),
      card?.querySelector(".tray-process-icon-slot"),
      titleButton.querySelector("span"),
      card?.querySelector(".tray-process-heading"),
      card,
      card?.querySelector(".tray-port-list"),
    ];

    for (const target of clickTargets) {
      expect(target).not.toBeNull();
      navigate.mockClear();
      fireEvent.click(target!);
      await waitFor(() => {
        expect(navigate).toHaveBeenCalledTimes(1);
        expect(navigate).toHaveBeenCalledWith({
          generation: 3,
          entryRef: "entry-card",
        });
      });
    }
  });

  it("keeps port and binding controls independent from card navigation", async () => {
    const navigate = vi.fn();
    mockIPC((command, payload) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(
          snapshot([
            entry("entry-3000-v4", 3000, "127.0.0.1"),
            entry("entry-3000-v6", 3000, "::1"),
            entry("entry-4000", 4000),
            entry("entry-5000", 5000),
            entry("entry-6000", 6000),
          ]),
        );
      if (command === "navigate_to_runtime_entry") navigate(payload);
      return undefined;
    });

    const { container } = render(<TrayPopover />);
    const showBindings = await screen.findByRole("button", {
      name: "Show bindings for TCP port 3000",
    });
    expect(
      container.querySelectorAll(".tray-process button button"),
    ).toHaveLength(0);
    showBindings.focus();
    expect(showBindings).toHaveFocus();

    fireEvent.click(showBindings);
    expect(
      screen.getByRole("button", { name: "Hide bindings for TCP port 3000" }),
    ).toHaveAttribute("aria-expanded", "true");
    expect(navigate).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole("button", { name: "Hide bindings for TCP port 3000" }),
    );
    expect(navigate).not.toHaveBeenCalled();

    const showMore = screen.getByRole("button", {
      name: "Show 1 more ports for node",
    });
    fireEvent.click(showMore);
    expect(navigate).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole("button", { name: "Show fewer ports for node" }),
    );
    expect(navigate).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole("button", { name: /Open TCP port 3000 for node/ }),
    );
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledTimes(1);
      expect(navigate).toHaveBeenCalledWith({
        generation: 3,
        entryRef: "entry-3000-v4",
      });
    });

    navigate.mockClear();
    fireEvent.click(
      screen.getByRole("button", { name: "Show bindings for TCP port 3000" }),
    );
    await screen.findByRole("region", { name: "TCP port 3000 bindings" });
    fireEvent.click(screen.getByRole("button", { name: /127\.0\.0\.1/ }));
    await waitFor(() => {
      expect(navigate).toHaveBeenCalledTimes(1);
      expect(navigate).toHaveBeenCalledWith({
        generation: 3,
        entryRef: "entry-3000-v4",
      });
    });
  });

  it("keeps one full-width binding panel per process and replaces it by port", async () => {
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(
            snapshot([
              entry("entry-5000-v4", 5000, "0.0.0.0"),
              entry("entry-5000-v6", 5000, "::"),
              entry("entry-7000-v4", 7000, "0.0.0.0"),
              entry("entry-7000-v6", 7000, "::"),
            ]),
          )
        : undefined,
    );
    const { container } = render(<TrayPopover />);

    await screen.findByRole("button", { name: /Open TCP port 5000/ });
    const firstDisclosure = screen.getByRole("button", {
      name: "Show bindings for TCP port 5000",
    });
    expect(firstDisclosure).not.toHaveAttribute("aria-controls");
    fireEvent.click(firstDisclosure);

    const firstPanel = screen.getByRole("region", {
      name: "TCP port 5000 bindings",
    });
    expect(firstPanel).toHaveTextContent("0.0.0.0");
    expect(firstPanel).toHaveTextContent("::");
    expect(firstDisclosure).toHaveAttribute("aria-expanded", "true");
    expect(firstDisclosure).toHaveAttribute("aria-controls", firstPanel.id);
    expect(firstPanel.closest(".tray-process")).toBeTruthy();
    expect(
      firstPanel.closest(".tray-process")?.querySelector(".tray-port-list"),
    ).not.toContainElement(firstPanel);

    fireEvent.click(
      screen.getByRole("button", {
        name: "Show bindings for TCP port 7000",
      }),
    );
    expect(
      screen.queryByRole("region", { name: "TCP port 5000 bindings" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "TCP port 7000 bindings" }),
    ).toBeInTheDocument();
    expect(container.querySelectorAll(".tray-bindings-panel")).toHaveLength(1);
  });

  it("keeps the selected bindings panel associated with a visible port", async () => {
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(
            snapshot([
              entry("entry-3000", 3000),
              entry("entry-4000", 4000),
              entry("entry-5000-a", 5000, "0.0.0.0"),
              entry("entry-5000-b", 5000, "::"),
              entry("entry-6000-a", 6000, "0.0.0.0"),
              entry("entry-6000-b", 6000, "::"),
            ]),
          )
        : undefined,
    );
    render(<TrayPopover />);

    await screen.findByRole("button", { name: /Open TCP port 5000/ });
    fireEvent.click(
      screen.getByRole("button", { name: "Show 1 more ports for node" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Show bindings for TCP port 6000" }),
    );
    expect(
      screen.getByRole("region", { name: "TCP port 6000 bindings" }),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Show fewer ports for node" }),
    );
    expect(
      screen.queryByRole("region", { name: "TCP port 6000 bindings" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Open TCP port 5000/ }),
    ).toBeInTheDocument();
  });

  it("invalidates binding disclosures when endpoints change in accepted snapshots", async () => {
    const originalEntries = [
      entry("entry-3000-a", 3000, "0.0.0.0"),
      entry("entry-3000-b", 3000, "::"),
      entry("entry-4000", 4000),
    ];
    let refreshCount = 0;
    mockIPC((command) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(snapshot(originalEntries, 3));
      if (command === "refresh_runtime_snapshot") {
        refreshCount += 1;
        return refreshCount === 1
          ? snapshot(
              [
                entry("entry-3000-a", 3000, "0.0.0.0"),
                entry("entry-4000", 4000),
              ],
              4,
            )
          : refreshCount === 2
            ? snapshot(originalEntries, 5)
            : refreshCount === 3
              ? snapshot([entry("entry-4000", 4000)], 6)
              : snapshot(originalEntries, 7);
      }
      return undefined;
    });
    render(<TrayPopover />);

    await screen.findByRole("button", { name: /Open TCP port 3000/ });
    fireEvent.click(
      screen.getByRole("button", { name: "Show bindings for TCP port 3000" }),
    );
    expect(
      screen.getByRole("region", { name: "TCP port 3000 bindings" }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Refresh runtimes" }));
    await screen.findByText("2 TCP endpoints · 1 identified process");
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Refresh runtimes" }),
      ).toBeEnabled(),
    );
    expect(
      screen.queryByRole("region", { name: "TCP port 3000 bindings" }),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Refresh runtimes" }));
    await screen.findByText("3 TCP endpoints · 1 identified process");
    expect(
      screen.queryByRole("region", { name: "TCP port 3000 bindings" }),
    ).not.toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Show bindings for TCP port 3000" }),
    );
    expect(
      screen.getByRole("region", { name: "TCP port 3000 bindings" }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Refresh runtimes" }));
    await screen.findByText("1 TCP endpoint · 1 identified process");
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Refresh runtimes" }),
      ).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Refresh runtimes" }));
    await screen.findByText("3 TCP endpoints · 1 identified process");
    expect(
      screen.queryByRole("region", { name: "TCP port 3000 bindings" }),
    ).not.toBeInTheDocument();
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

  it("shows three ports initially and reveals remaining ports in the same badge list", async () => {
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(
            snapshot([
              entry("entry-a", 3000),
              entry("entry-b", 4000),
              entry("entry-c", 5000),
              entry("entry-d", 6000),
              entry("entry-e", 7000),
            ]),
          )
        : undefined,
    );
    const { container } = render(<TrayPopover />);

    expect(
      await screen.findByRole("button", { name: /Open TCP port 5000/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Open TCP port 6000/ }),
    ).not.toBeInTheDocument();
    const list = container.querySelector(".tray-port-list");
    expect(list?.querySelectorAll(".tray-port-chip")).toHaveLength(3);
    fireEvent.click(
      screen.getByRole("button", { name: "Show 2 more ports for node" }),
    );
    expect(
      screen.getByRole("button", { name: /Open TCP port 6000/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Open TCP port 7000/ }),
    ).toBeInTheDocument();
    expect(list?.querySelectorAll(".tray-port-chip")).toHaveLength(5);
    expect(
      screen.getByRole("button", { name: "Show fewer ports for node" }),
    ).toHaveAttribute("aria-expanded", "true");
  });

  it("keeps port expansion local to the process instance", async () => {
    const otherProcess = entry("entry-other-a", 8000);
    otherProcess.processId = 21;
    otherProcess.process.details.processId = 21;
    otherProcess.process.details.startTimeUnixMs.value = 200;
    otherProcess.process.details.name.value = "python";
    const otherPorts = [9000, 10000, 11000].map((port) => ({
      ...otherProcess,
      entryRef: `entry-other-${port}`,
      port,
    }));
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(
            snapshot([
              entry("entry-a", 3000),
              entry("entry-b", 4000),
              entry("entry-c", 5000),
              entry("entry-d", 6000),
              entry("entry-e", 7000),
              otherProcess,
              ...otherPorts,
            ]),
          )
        : undefined,
    );
    render(<TrayPopover />);

    await screen.findByRole("button", { name: /Open TCP port 3000/ });
    fireEvent.click(
      screen.getByRole("button", { name: "Show 2 more ports for node" }),
    );
    expect(
      screen.getByRole("button", { name: /Open TCP port 6000/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Open TCP port 11000/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Show 1 more ports for python" }),
    ).toHaveAttribute("aria-expanded", "false");
  });

  it("loads trusted process icons by accepted snapshot generation and keeps fallback when absent", async () => {
    const processEntry = {
      ...entry("entry-a", 3000),
      processIconRef: "icon-ref-1",
    };
    mockIPC((command, payload) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(snapshot([processEntry], 9));
      if (command === "get_runtime_process_icons") {
        expect(payload).toEqual({ generation: 9 });
        return [{ reference: "icon-ref-1", pngBase64: "aWNvbg==" }];
      }
      return undefined;
    });

    const { container } = render(<TrayPopover />);
    await screen.findByRole("button", { name: /Open TCP port 3000/ });
    await waitFor(() =>
      expect(container.querySelector(".tray-process img")).toHaveAttribute(
        "src",
        "data:image/png;base64,aWNvbg==",
      ),
    );
  });

  it("shows a stable generic icon slot when the backend has no icon asset", async () => {
    const processEntry = {
      ...entry("entry-a", 3000),
      processIconRef: "missing-icon-ref",
    };
    mockIPC((command) =>
      command === "get_tray_runtime_snapshot"
        ? trayState(snapshot([processEntry]))
        : command === "get_runtime_process_icons"
          ? []
          : undefined,
    );

    const { container } = render(<TrayPopover />);
    await screen.findByRole("button", { name: /Open TCP port 3000/ });
    await waitFor(() =>
      expect(
        container.querySelector(
          ".tray-process-icon-slot .fallback-icon[data-icon-kind='process-fallback']",
        ),
      ).toBeInTheDocument(),
    );
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
    const dismiss = vi.fn();
    mockIPC((command, payload) => {
      if (command === "get_tray_runtime_snapshot")
        return trayState(snapshot([entry("entry-a", 3000)], 4));
      if (command === "navigate_to_runtime_entry") navigate(payload);
      if (command === "dismiss_tray_popover") dismiss();
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
    await waitFor(() => expect(dismiss).toHaveBeenCalledOnce());
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

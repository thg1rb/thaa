import { beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { runtimeClient } from "./runtimeClient";

describe("runtimeClient project terminal action", () => {
  beforeEach(() => invokeMock.mockReset());

  it("sends only the current snapshot entry reference", async () => {
    await runtimeClient.openProjectInTerminal("entry-7-a");

    expect(invokeMock).toHaveBeenCalledExactlyOnceWith(
      "open_project_in_terminal",
      { entryRef: "entry-7-a" },
    );
  });

  it("preserves backend errors for the action hook", async () => {
    invokeMock.mockRejectedValueOnce(new Error("stale entry"));

    await expect(
      runtimeClient.openProjectInTerminal("entry-7-a"),
    ).rejects.toThrow("stale entry");
  });
});

describe("runtimeClient project editor action", () => {
  beforeEach(() => invokeMock.mockReset());

  it("sends only the current snapshot entry reference", async () => {
    await runtimeClient.openProjectInEditor("entry-7-a");

    expect(invokeMock).toHaveBeenCalledExactlyOnceWith(
      "open_project_in_editor",
      { entryRef: "entry-7-a" },
    );
  });

  it("preserves backend errors for the action hook", async () => {
    invokeMock.mockRejectedValueOnce(new Error("stale entry"));

    await expect(
      runtimeClient.openProjectInEditor("entry-7-a"),
    ).rejects.toThrow("stale entry");
  });
});

describe("runtimeClient tray navigation", () => {
  beforeEach(() => invokeMock.mockReset());

  it("requests only an existing accepted snapshot without triggering a scan", async () => {
    invokeMock.mockResolvedValueOnce({ snapshot: null, stale: false });

    await expect(runtimeClient.getTrayRuntimeSnapshot()).resolves.toEqual({
      snapshot: null,
      stale: false,
    });
    expect(invokeMock).toHaveBeenCalledExactlyOnceWith(
      "get_tray_runtime_snapshot",
    );
  });

  it("binds navigation to the accepted generation and opaque entry reference", async () => {
    await runtimeClient.navigateToRuntimeEntry(12, "entry-c-2");

    expect(invokeMock).toHaveBeenCalledExactlyOnceWith(
      "navigate_to_runtime_entry",
      { generation: 12, entryRef: "entry-c-2" },
    );
  });

  it("opens the existing main window through its narrow command", async () => {
    await runtimeClient.showMainWindow();

    expect(invokeMock).toHaveBeenCalledExactlyOnceWith("show_main_window");
  });

  it("dismisses the tray popover through its window-scoped command", async () => {
    await runtimeClient.dismissTrayPopover();

    expect(invokeMock).toHaveBeenCalledExactlyOnceWith("dismiss_tray_popover");
  });
});

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

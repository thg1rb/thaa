import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ToastInput } from "../../../shared/ui/toast/toastTypes";

const openProjectInTerminal = vi.hoisted(() => vi.fn());

vi.mock("../api/runtimeClient", () => ({
  runtimeClient: { openProjectInTerminal },
}));

import { useRuntimeEntryActions } from "./useRuntimeEntryActions";

describe("useRuntimeEntryActions terminal action", () => {
  beforeEach(() => openProjectInTerminal.mockReset());

  it("requests the selected snapshot entry", async () => {
    const showToast = vi.fn<(toast: ToastInput) => void>();
    const { result } = renderHook(() => useRuntimeEntryActions(showToast));

    await act(async () => {
      await result.current.openProjectInTerminal("entry-3-2");
    });

    expect(openProjectInTerminal).toHaveBeenCalledExactlyOnceWith("entry-3-2");
    expect(showToast).not.toHaveBeenCalled();
  });

  it("shows the backend's safe recoverable failure message", async () => {
    openProjectInTerminal.mockRejectedValueOnce(
      "macOS denied Thaa permission to control Terminal. Allow it in System Settings > Privacy & Security > Automation, then try again.",
    );
    const showToast = vi.fn<(toast: ToastInput) => void>();
    const { result } = renderHook(() => useRuntimeEntryActions(showToast));

    await act(async () => {
      await result.current.openProjectInTerminal("entry-3-2");
    });

    expect(showToast).toHaveBeenCalledExactlyOnceWith({
      tone: "error",
      title: "Could not open project in Terminal",
      message:
        "macOS denied Thaa permission to control Terminal. Allow it in System Settings > Privacy & Security > Automation, then try again.",
    });
  });
});

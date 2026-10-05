import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { useRef } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ToastProvider } from "./ToastProvider";
import { useToast } from "./useToast";

function ToastControls() {
  const { showToast } = useToast();
  const notificationCount = useRef(0);
  return (
    <div>
      <button
        onClick={() => {
          notificationCount.current += 1;
          showToast({
            tone: "success",
            title: "Request sent",
            message: `Notification ${notificationCount.current}: The process result will be checked on the next scan.`,
          });
        }}
      >
        Add success
      </button>
      <button
        onClick={() =>
          showToast({
            tone: "error",
            title: "Action failed",
            message: "Permission was denied.",
          })
        }
      >
        Add error
      </button>
    </div>
  );
}

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("ToastProvider", () => {
  it("announces success and error with their respective live-region roles", () => {
    render(
      <ToastProvider>
        <ToastControls />
      </ToastProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "Add success" }));
    expect(screen.getByRole("status")).toHaveTextContent("Request sent");
    expect(screen.getByText(/checked on the next scan/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Add error" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Action failed");
  });

  it("animates manual dismissal before promoting a queued notification", () => {
    render(
      <ToastProvider>
        <ToastControls />
      </ToastProvider>,
    );
    const add = screen.getByRole("button", { name: "Add success" });
    fireEvent.click(add);
    fireEvent.click(add);
    fireEvent.click(add);
    fireEvent.click(add);

    expect(screen.getAllByRole("status")).toHaveLength(3);
    fireEvent.click(
      screen.getAllByRole("button", {
        name: "Dismiss notification: Request sent",
      })[0]!,
    );
    expect(screen.getAllByRole("status")).toHaveLength(3);
    const exiting = screen.getAllByRole("status")[0]!;
    expect(exiting).toHaveClass("toast-exiting");
    fireEvent.animationEnd(exiting, { animationName: "toast-out" });
    expect(screen.getAllByRole("status")).toHaveLength(3);
  });

  it("keeps a timed-out notification mounted until its exit animation ends", () => {
    vi.useFakeTimers();
    render(
      <ToastProvider>
        <ToastControls />
      </ToastProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Add success" }));
    expect(screen.getByRole("status")).toBeInTheDocument();

    act(() => vi.advanceTimersByTime(6000));
    const exiting = screen.getByRole("status");
    expect(exiting).toHaveClass("toast-exiting");
    expect(exiting).toBeInTheDocument();
    fireEvent.animationEnd(exiting, { animationName: "toast-out" });
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("keeps the newest notification at the bottom and queues while exits finish", () => {
    render(
      <ToastProvider>
        <ToastControls />
      </ToastProvider>,
    );
    const add = screen.getByRole("button", { name: "Add success" });
    fireEvent.click(add);
    fireEvent.click(add);
    fireEvent.click(add);
    fireEvent.click(add);
    const viewport = screen.getByRole("list", { name: "Notifications" });
    expect(viewport).toHaveClass("toast-viewport");
    expect(viewport.lastElementChild).toHaveTextContent("Notification 3");
    const first = screen.getAllByRole("status")[0]!;
    fireEvent.click(
      within(first).getByRole("button", { name: /Dismiss notification/ }),
    );
    fireEvent.click(add);
    expect(screen.getAllByRole("status")).toHaveLength(3);
    expect(first).toHaveClass("toast-exiting");
    fireEvent.animationEnd(first, { animationName: "toast-out" });
    expect(screen.getAllByRole("status")).toHaveLength(3);
    expect(viewport.lastElementChild).toHaveTextContent("Notification 4");
  });
});

import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ToastProvider } from "./ToastProvider";
import { useToast } from "./useToast";

function ToastControls() {
  const { showToast } = useToast();
  return (
    <div>
      <button
        onClick={() =>
          showToast({
            tone: "success",
            title: "Request sent",
            message: "The process result will be checked on the next scan.",
          })
        }
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

  it("supports manual dismissal and promotes queued notifications", () => {
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
  });

  it("automatically dismisses a success notification after its reading window", () => {
    vi.useFakeTimers();
    render(
      <ToastProvider>
        <ToastControls />
      </ToastProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Add success" }));
    expect(screen.getByRole("status")).toBeInTheDocument();

    act(() => vi.advanceTimersByTime(6000));
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });
});

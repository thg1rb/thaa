import { cleanup, render, screen } from "@testing-library/react";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it } from "vitest";
import App from "./App";

afterEach(() => {
  cleanup();
  clearMocks();
});

describe("application shell", () => {
  it("shows a loading status while app information is pending", () => {
    mockIPC(() => new Promise(() => undefined));

    render(<App />);

    expect(screen.getByRole("status")).toHaveTextContent(
      "Connecting to the local application",
    );
  });

  it("renders the typed application information response", async () => {
    mockIPC((command) => {
      if (command === "get_app_info") {
        return {
          productName: "Thaa — Local Runtime Inspector",
          version: "0.1.0",
        };
      }
      throw new Error("Unexpected IPC command");
    });

    render(<App />);

    expect(
      await screen.findByText("Thaa — Local Runtime Inspector"),
    ).toBeInTheDocument();
    expect(screen.getByText("Version 0.1.0")).toBeInTheDocument();
    expect(
      screen.getByText(/Runtime inspection will be added/),
    ).toBeInTheDocument();
  });

  it("shows a safe error when the backend request fails", async () => {
    mockIPC(() => Promise.reject("sensitive internal detail"));

    render(<App />);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Thaa could not connect to its local application service",
    );
    expect(
      screen.queryByText("sensitive internal detail"),
    ).not.toBeInTheDocument();
  });
});

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type AppInfo = {
  productName: string;
  version: string;
};

type LoadState =
  | { status: "loading" }
  | { status: "ready"; info: AppInfo }
  | { status: "error" };

export default function App() {
  const [state, setState] = useState<LoadState>({ status: "loading" });

  useEffect(() => {
    let active = true;

    invoke<AppInfo>("get_app_info")
      .then((info) => {
        if (active) setState({ status: "ready", info });
      })
      .catch(() => {
        if (active) setState({ status: "error" });
      });

    return () => {
      active = false;
    };
  }, []);

  return (
    <main className="shell">
      <header className="brand">
        <span className="brand-mark" aria-hidden="true">
          T
        </span>
        <div>
          <p className="eyebrow">LOCAL RUNTIME INSPECTOR</p>
          <h1>Thaa</h1>
        </div>
      </header>

      <section className="status-card" aria-labelledby="status-title">
        <p className="eyebrow">APPLICATION STATUS</p>
        <h2 id="status-title">Your development shell is ready</h2>
        <p className="description">
          Runtime inspection will be added in later work items.
        </p>

        {state.status === "loading" && (
          <p className="response" role="status">
            Connecting to the local application…
          </p>
        )}
        {state.status === "ready" && (
          <div className="response" role="status">
            <span>{state.info.productName}</span>
            <span className="version">Version {state.info.version}</span>
          </div>
        )}
        {state.status === "error" && (
          <p className="response error" role="alert">
            Thaa could not connect to its local application service. Restart the
            app and try again.
          </p>
        )}
      </section>
      <footer>All application data stays on this device.</footer>
    </main>
  );
}

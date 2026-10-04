import { lazy, Suspense } from "react";
import { RuntimeInspectorPage } from "./features/runtime-inspector/RuntimeInspectorPage";
import { ToastProvider } from "./shared/ui/toast/ToastProvider";

const RuntimeVisualFixture = import.meta.env.DEV
  ? lazy(() =>
      import("./dev/RuntimeVisualFixture").then((module) => ({
        default: module.RuntimeVisualFixture,
      })),
    )
  : null;

function AppContent() {
  const showVisualFixture =
    import.meta.env.DEV &&
    new URLSearchParams(window.location.search).get("w0131") === "preview";

  if (showVisualFixture && RuntimeVisualFixture) {
    return (
      <Suspense fallback={<main className="app-shell" aria-busy="true" />}>
        <RuntimeVisualFixture />
      </Suspense>
    );
  }
  return <RuntimeInspectorPage />;
}

export default function App() {
  return (
    <ToastProvider>
      <AppContent />
    </ToastProvider>
  );
}

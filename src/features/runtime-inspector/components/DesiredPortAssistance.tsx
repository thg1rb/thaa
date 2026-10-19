import type { DesiredPortAssistance as DesiredPortResult } from "../utils/desiredPortAssistance";

export function DesiredPortAssistance({
  result,
  partial,
  stale,
  hasAdditionalNameMatches,
}: {
  result: DesiredPortResult;
  partial: boolean;
  stale: boolean;
  hasAdditionalNameMatches: boolean;
}) {
  const { port, matchingEntries } = result;
  const count = matchingEntries.length;

  return (
    <section
      className="port-assistance"
      role="region"
      aria-label={`Observed TCP listener results for port ${port}`}
    >
      {count > 0 ? (
        <>
          <p className="port-assistance-summary">
            Observed {count} TCP {count === 1 ? "listener" : "listeners"} for
            port {port} in the displayed snapshot.
          </p>
          <p>
            A matching listener may be relevant to a binding conflict, but this
            observation cannot establish whether another bind will fail.
          </p>
        </>
      ) : (
        <>
          <p className="port-assistance-summary">
            No TCP listener for port {port} was observed in the displayed
            snapshot.
          </p>
          <p>This does not guarantee that the port is available.</p>
        </>
      )}
      {hasAdditionalNameMatches && (
        <p>
          Search also includes process-name matches; additional rows may not be
          listeners observed on port {port}.
        </p>
      )}
      {partial && <p>The scan was partial; some listeners may be missing.</p>}
      {stale && <p>The displayed snapshot may be out of date.</p>}
    </section>
  );
}

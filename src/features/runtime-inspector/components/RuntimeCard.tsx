import type { Action, RuntimeEntry, RuntimeSnapshot } from "../model/types";
import {
  availableText,
  displayName,
  safeDisplay,
} from "../utils/processDisplay";
import { BindingBadge } from "./BindingBadge";
import { ProcessIcon } from "./ProcessIcon";
import { RuntimeActions } from "./RuntimeActions";
import {
  formatCpuPercent,
  formatMemoryBytes,
  formatUptime,
} from "../utils/resourceMetrics";

export function RuntimeCard({
  entry,
  capabilities,
  iconSource,
  iconLoading,
  busy,
  onAction,
  onConfirmForce,
  onOpen,
  onCopy,
}: {
  entry: RuntimeEntry;
  capabilities: RuntimeSnapshot["capabilities"];
  iconSource: string | undefined;
  iconLoading: boolean;
  busy: boolean;
  onAction: (targetRef: string, action: Action) => void;
  onConfirmForce: () => void;
  onOpen: () => void;
  onCopy: (value: string, label: string) => void;
}) {
  const info =
    entry.process.state === "available" ? entry.process.details : null;
  const processName = displayName(entry);
  const executable = info ? availableText(info.executablePath) : null;
  const workingDirectory = info ? availableText(info.workingDirectory) : null;
  const projectRoot = entry.projectRoot ? safeDisplay(entry.projectRoot) : null;
  const gitRepository = entry.gitContext
    ? safeDisplay(entry.gitContext.repositoryRoot)
    : null;
  const gitBranch = entry.gitContext
    ? entry.gitContext.branch.state === "named"
      ? safeDisplay(entry.gitContext.branch.name)
      : "Detached HEAD"
    : null;
  const resourceMetrics = entry.resourceMetrics;
  const pid = entry.processId;

  return (
    <article className="runtime-card">
      <div className="runtime-details">
        <div className="process-heading">
          <ProcessIcon
            source={iconSource}
            unknown={entry.process.state === "noOwner"}
            loading={iconLoading}
          />
          <div className="process-title">
            <p className="eyebrow">PROCESS / OWNER</p>
            <h3 title={processName}>{processName}</h3>
            <p>
              {entry.process.state === "noOwner"
                ? "Owner not identified"
                : `PID ${pid ?? "unknown"}`}
              <span className="separator">·</span>
              {entry.localAddress ?? "Address unavailable"}
            </p>
          </div>
          <div className="port-identity">
            <span className="eyebrow">TCP PORT</span>
            <strong className="port-number">:{entry.port}</strong>
          </div>
          <div className="entry-actions">
            {entry.localUrl && (
              <button
                className="icon-button"
                aria-label={`Open ${entry.localUrl}`}
                title="Open local service"
                onClick={onOpen}
              >
                ↗
              </button>
            )}
            {entry.localUrl && (
              <button
                className="icon-button"
                aria-label={`Copy URL for port ${entry.port}`}
                title="Copy URL"
                onClick={() => onCopy(entry.localUrl!, "URL")}
              >
                ⧉
              </button>
            )}
            <button
              className="icon-button copy-value"
              aria-label={`Copy port ${entry.port}`}
              title="Copy port"
              onClick={() => onCopy(String(entry.port), "Port")}
            >
              #
            </button>
            {pid !== null && (
              <button
                className="icon-button copy-value"
                aria-label={`Copy PID ${pid}`}
                title="Copy PID"
                onClick={() => onCopy(String(pid), "PID")}
              >
                ID
              </button>
            )}
          </div>
        </div>
        <div className="metadata-row">
          {resourceMetrics && (
            <>
              <span title="CPU normalized to total logical CPU capacity">
                CPU · {formatCpuPercent(resourceMetrics.cpuPercent)}
              </span>
              <span title="Resident memory / working set">
                Memory · {formatMemoryBytes(resourceMetrics.memoryBytes)}
              </span>
              <span title="Time since the process start time">
                Uptime · {formatUptime(resourceMetrics.uptimeMs)}
              </span>
            </>
          )}
          {executable && <span title={executable}>{executable}</span>}
          {workingDirectory && (
            <span title={workingDirectory}>in {workingDirectory}</span>
          )}
          {projectRoot && (
            <span title={projectRoot}>Project root · {projectRoot}</span>
          )}
          {gitRepository && gitBranch && (
            <span title={gitRepository + " · " + gitBranch}>
              Git · {gitRepository} · {gitBranch}
            </span>
          )}
          {info && info.name.state !== "available" && (
            <span className="muted-metadata">Process name unavailable</span>
          )}
          {!info && (
            <span className="muted-metadata">
              {entry.process.state === "unavailable"
                ? safeDisplay(entry.process.details.reason)
                : "Owner not identified"}
            </span>
          )}
          {info &&
            !executable &&
            !workingDirectory &&
            !projectRoot &&
            !resourceMetrics && (
              <span className="muted-metadata">
                Additional process details unavailable
              </span>
            )}
        </div>
        <div className="card-bottom">
          <BindingBadge
            binding={entry.binding}
            localAddress={entry.localAddress}
          />
          <RuntimeActions
            targetRef={entry.actionTargetRef}
            capabilities={capabilities}
            busy={busy}
            onAction={onAction}
            onConfirmForce={onConfirmForce}
          />
        </div>
      </div>
    </article>
  );
}

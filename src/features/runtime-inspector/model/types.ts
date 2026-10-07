export type Field<T> =
  { state: "available"; value: T } | { state: "unavailable"; value: string };

export type ProcessInfo = {
  processId: number;
  name: Field<string>;
  executablePath: Field<string>;
  startTimeUnixMs: Field<number>;
  commandArguments: Field<string[]>;
  workingDirectory: Field<string>;
};

export type ProcessDetails =
  | { state: "noOwner" }
  | { state: "available"; details: ProcessInfo }
  | { state: "unavailable"; details: { processId: number; reason: string } };

export type RuntimeEntry = {
  entryRef: string;
  protocol: "tcp";
  localAddress: string | null;
  port: number;
  binding:
    | "loopbackOnly"
    | "wildcardIpv4"
    | "wildcardIpv6"
    | "specificAddress"
    | "unknown";
  processId: number | null;
  parentProcessId: number | null;
  process: ProcessDetails;
  resourceMetrics: {
    cpuPercent: number | null;
    memoryBytes: number | null;
    uptimeMs: number | null;
  } | null;
  projectRoot: string | null;
  gitContext: GitContext | null;
  localUrl: string | null;
  actionTargetRef: string | null;
  processIconRef: string | null;
};

export type GitContext = {
  repositoryRoot: string;
  branch: { state: "named"; name: string } | { state: "detachedHead" };
};

export type ProcessIconAsset = { reference: string; pngBase64: string };

export type RuntimeSnapshot = {
  generation: number;
  observedAtUnixMs: number;
  completeness: { state: "complete" } | { state: "partial"; reason: string };
  capabilities: { gracefulStop: boolean; forceStop: boolean };
  entries: RuntimeEntry[];
  processIcons: ProcessIconAsset[];
};

export type RuntimeSnapshotDto = Omit<RuntimeSnapshot, "processIcons">;
export type RuntimeError = { state: string; message?: string };
export type Action = "gracefulStop" | "forceStop";
export type ActionResult =
  | { state: "requested" }
  | { state: "alreadyExited" }
  | { state: "refused"; reason: string }
  | { state: "failed"; reason: string };

export type RefreshSource = "initial" | "manual" | "background" | "action";
export type RefreshOutcome = "updated" | "failed" | "queued";

export type RuntimeInspectorState = {
  snapshot: RuntimeSnapshot | null;
  loading: boolean;
  iconsLoading: boolean;
  refreshing: boolean;
  error: string | null;
  backgroundStale: boolean;
};

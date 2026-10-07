export function formatCpuPercent(value: number | null): string {
  if (value === null || !Number.isFinite(value) || value < 0) return "—";
  const rounded = Math.round(value * 10) / 10;
  return `${Number.isInteger(rounded) ? rounded.toFixed(0) : rounded.toFixed(1)}%`;
}

export function formatMemoryBytes(value: number | null): string {
  if (value === null || !Number.isSafeInteger(value) || value < 0) return "—";
  if (value < 1024) return `${value} B`;

  const units = ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
  let amount = value;
  let unit = "B";
  for (const nextUnit of units) {
    amount /= 1024;
    unit = nextUnit;
    if (amount < 1024 || nextUnit === "EiB") break;
  }
  return `${amount >= 10 ? amount.toFixed(0) : amount.toFixed(1)} ${unit}`;
}

export function formatUptime(milliseconds: number | null): string {
  if (
    milliseconds === null ||
    !Number.isSafeInteger(milliseconds) ||
    milliseconds < 0
  ) {
    return "—";
  }

  const totalSeconds = Math.floor(milliseconds / 1000);
  if (totalSeconds < 60) return `${totalSeconds}s`;
  const totalMinutes = Math.floor(totalSeconds / 60);
  if (totalMinutes < 60) return `${totalMinutes}m`;
  const totalHours = Math.floor(totalMinutes / 60);
  if (totalHours < 24) {
    return `${totalHours}h${totalMinutes % 60 ? ` ${totalMinutes % 60}m` : ""}`;
  }

  const days = Math.floor(totalHours / 24);
  const hours = totalHours % 24;
  return `${days}d${hours ? ` ${hours}h` : ""}`;
}

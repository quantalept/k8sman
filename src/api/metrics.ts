import { invoke } from "@tauri-apps/api/core";

export function getPodMetrics(contextName: string, namespace?: string): Promise<any[]> {
  return invoke("get_pod_metrics", { contextName, namespace });
}

export function getNodeMetrics(contextName: string): Promise<any[]> {
  return invoke("get_node_metrics", { contextName });
}

/** Parses a Kubernetes CPU quantity ("500m", "2", "480800u", "3750933n") into cores. */
export function parseCpuQuantity(value: string | undefined): number {
  if (!value) return 0;
  const match = value.match(/^(\d+(?:\.\d+)?)([num]?)$/);
  if (!match) return 0;
  const [, num, suffix] = match;
  const n = parseFloat(num);
  switch (suffix) {
    case "n":
      return n / 1e9;
    case "u":
      return n / 1e6;
    case "m":
      return n / 1e3;
    default:
      return n;
  }
}

const MEMORY_UNITS: Record<string, number> = {
  Ki: 1024,
  Mi: 1024 ** 2,
  Gi: 1024 ** 3,
  Ti: 1024 ** 4,
  K: 1000,
  M: 1000 ** 2,
  G: 1000 ** 3,
  T: 1000 ** 4,
};

/** Parses a Kubernetes memory quantity ("7344360Ki", "512Mi", "1000000") into bytes. */
export function parseMemoryQuantity(value: string | undefined): number {
  if (!value) return 0;
  const match = value.match(/^(\d+(?:\.\d+)?)([A-Za-z]*)$/);
  if (!match) return 0;
  const [, num, suffix] = match;
  const n = parseFloat(num);
  if (!suffix) return n;
  return n * (MEMORY_UNITS[suffix] ?? 1);
}

/** Formats a core count as millicores below 1 core, cores otherwise ("120m", "1.4"). */
export function formatCores(cores: number): string {
  if (cores === 0) return "0m";
  if (cores < 1) return `${Math.round(cores * 1000)}m`;
  return `${cores.toFixed(2)}`;
}

/** Formats a byte count as the largest binary unit that keeps it >= 1 ("128Mi", "1.3Gi"). */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0Mi";
  const units = ["Ki", "Mi", "Gi", "Ti"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(value >= 10 ? 0 : 1)}${units[unitIndex]}`;
}

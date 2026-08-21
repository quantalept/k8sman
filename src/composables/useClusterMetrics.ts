import { ref, watch, onScopeDispose } from "vue";
import {
  getNodeMetrics,
  getPrometheusUrl,
  queryPrometheusRange,
  parseCpuQuantity,
  parseMemoryQuantity,
} from "../api/metrics";
import { listResources } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

const POLL_INTERVAL_MS = 10_000;
const MAX_SAMPLES = 30;

// Standard kube-prometheus-stack-style queries for cluster-wide container CPU/memory. Only
// exercised when a Prometheus URL is configured in Settings.
const CPU_PROMQL = 'sum(rate(container_cpu_usage_seconds_total{container!="",container!="POD"}[5m]))';
const MEMORY_PROMQL = 'sum(container_memory_working_set_bytes{container!="",container!="POD"})';

export interface MetricSample {
  timestamp: number;
  cpuCores: number;
  cpuPercent: number;
  memoryBytes: number;
  memoryPercent: number;
}

export function useClusterMetrics() {
  const cluster = useClusterStore();
  const samples = ref<MetricSample[]>([]);
  const error = ref<string | null>(null);
  const supported = ref(true);

  let allocatableCpu = 0;
  let allocatableMemory = 0;
  let prometheusUrl: string | undefined;
  let timer: ReturnType<typeof setInterval> | null = null;

  async function loadAllocatable(contextName: string) {
    const nodes = await listResources(contextName, "Node");
    allocatableCpu = nodes.reduce(
      (sum, n) => sum + parseCpuQuantity(n.status?.allocatable?.cpu),
      0,
    );
    allocatableMemory = nodes.reduce(
      (sum, n) => sum + parseMemoryQuantity(n.status?.allocatable?.memory),
      0,
    );
  }

  async function pollPrometheus() {
    const stepSeconds = POLL_INTERVAL_MS / 1000;
    const end = Math.floor(Date.now() / 1000);
    const start = end - MAX_SAMPLES * stepSeconds;

    const [cpuPoints, memPoints] = await Promise.all([
      queryPrometheusRange(prometheusUrl!, CPU_PROMQL, start, end, stepSeconds),
      queryPrometheusRange(prometheusUrl!, MEMORY_PROMQL, start, end, stepSeconds),
    ]);
    const memByTimestamp = new Map(memPoints.map((p) => [p.timestamp, p.value]));

    samples.value = cpuPoints.slice(-MAX_SAMPLES).map((p) => {
      const memoryBytes = memByTimestamp.get(p.timestamp) ?? 0;
      return {
        timestamp: p.timestamp * 1000,
        cpuCores: p.value,
        cpuPercent: allocatableCpu ? (p.value / allocatableCpu) * 100 : 0,
        memoryBytes,
        memoryPercent: allocatableMemory ? (memoryBytes / allocatableMemory) * 100 : 0,
      };
    });
  }

  async function pollMetricsServer() {
    const contextName = cluster.currentContext;
    if (!contextName) return;
    const metrics = await getNodeMetrics(contextName);
    const cpuCores = metrics.reduce((sum, m) => sum + parseCpuQuantity(m.usage?.cpu), 0);
    const memoryBytes = metrics.reduce(
      (sum, m) => sum + parseMemoryQuantity(m.usage?.memory),
      0,
    );
    samples.value = [
      ...samples.value.slice(-(MAX_SAMPLES - 1)),
      {
        timestamp: Date.now(),
        cpuCores,
        cpuPercent: allocatableCpu ? (cpuCores / allocatableCpu) * 100 : 0,
        memoryBytes,
        memoryPercent: allocatableMemory ? (memoryBytes / allocatableMemory) * 100 : 0,
      },
    ];
  }

  async function poll() {
    const contextName = cluster.currentContext;
    if (!contextName) return;
    try {
      if (allocatableCpu === 0) await loadAllocatable(contextName);
      if (prometheusUrl) {
        await pollPrometheus();
      } else {
        await pollMetricsServer();
      }
      error.value = null;
      supported.value = true;
    } catch (e) {
      error.value = String(e);
      // Neither metrics-server nor the configured Prometheus is reachable - stop polling a
      // failing endpoint rather than hammering it.
      supported.value = false;
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
    }
  }

  async function restart() {
    if (timer) clearInterval(timer);
    samples.value = [];
    allocatableCpu = 0;
    allocatableMemory = 0;
    prometheusUrl = undefined;
    const contextName = cluster.currentContext;
    if (!contextName) return;
    prometheusUrl = (await getPrometheusUrl(contextName)) || undefined;
    poll();
    timer = setInterval(poll, POLL_INTERVAL_MS);
  }

  watch(() => cluster.currentContext, restart, { immediate: true });

  onScopeDispose(() => {
    if (timer) clearInterval(timer);
  });

  return { samples, error, supported };
}

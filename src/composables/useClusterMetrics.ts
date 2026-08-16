import { ref, watch, onScopeDispose } from "vue";
import { getNodeMetrics, parseCpuQuantity, parseMemoryQuantity } from "../api/metrics";
import { listResources } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

const POLL_INTERVAL_MS = 10_000;
const MAX_SAMPLES = 30;

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

  async function poll() {
    const contextName = cluster.currentContext;
    if (!contextName) return;
    try {
      if (allocatableCpu === 0) await loadAllocatable(contextName);
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
      error.value = null;
      supported.value = true;
    } catch (e) {
      error.value = String(e);
      // metrics-server not installed, or no permission - stop polling a failing endpoint.
      supported.value = false;
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
    }
  }

  function restart() {
    if (timer) clearInterval(timer);
    samples.value = [];
    allocatableCpu = 0;
    allocatableMemory = 0;
    if (!cluster.currentContext) return;
    poll();
    timer = setInterval(poll, POLL_INTERVAL_MS);
  }

  watch(() => cluster.currentContext, restart, { immediate: true });

  onScopeDispose(() => {
    if (timer) clearInterval(timer);
  });

  return { samples, error, supported };
}

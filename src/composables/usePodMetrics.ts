import { ref, watch, onScopeDispose, type Ref } from "vue";
import { getPodMetrics, parseCpuQuantity, parseMemoryQuantity } from "../api/metrics";
import { useClusterStore } from "../stores/cluster";

const POLL_INTERVAL_MS = 10_000;

export interface ContainerUsage {
  name: string;
  cpuCores: number;
  memoryBytes: number;
}

export interface PodUsage {
  cpuCores: number;
  memoryBytes: number;
  containers: ContainerUsage[];
}

/** Live pod CPU/memory usage from metrics-server, keyed by "namespace/name", polled every 10s. */
export function usePodMetrics(namespace?: Ref<string | undefined>) {
  const cluster = useClusterStore();
  const usage = ref<Map<string, PodUsage>>(new Map());
  const error = ref<string | null>(null);
  const supported = ref(true);

  let timer: ReturnType<typeof setInterval> | null = null;

  async function poll() {
    const contextName = cluster.currentContext;
    if (!contextName) return;
    try {
      const metrics = await getPodMetrics(contextName, namespace?.value);
      const next = new Map<string, PodUsage>();
      for (const m of metrics) {
        const containers: ContainerUsage[] = (m.containers ?? []).map((c: any) => ({
          name: c.name,
          cpuCores: parseCpuQuantity(c.usage?.cpu),
          memoryBytes: parseMemoryQuantity(c.usage?.memory),
        }));
        const key = `${m.metadata.namespace}/${m.metadata.name}`;
        next.set(key, {
          cpuCores: containers.reduce((sum, c) => sum + c.cpuCores, 0),
          memoryBytes: containers.reduce((sum, c) => sum + c.memoryBytes, 0),
          containers,
        });
      }
      usage.value = next;
      error.value = null;
      supported.value = true;
    } catch (e) {
      error.value = String(e);
      supported.value = false;
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
    }
  }

  function restart() {
    if (timer) clearInterval(timer);
    if (!cluster.currentContext) return;
    poll();
    timer = setInterval(poll, POLL_INTERVAL_MS);
  }

  watch(() => [cluster.currentContext, namespace?.value], restart, { immediate: true });

  onScopeDispose(() => {
    if (timer) clearInterval(timer);
  });

  return { usage, error, supported };
}

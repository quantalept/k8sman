import { ref, watch, onScopeDispose, toValue, type MaybeRefOrGetter, type Ref } from "vue";
import { listResources, startWatch, type ResourceEvent } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

function objectKey(obj: any): string {
  const meta = obj?.metadata ?? {};
  return meta.uid ?? `${meta.namespace ?? ""}/${meta.name ?? ""}`;
}

// A backend watch's initial sync re-delivers every already-existing object as its own
// event on top of the bulk `listResources` call that already fetched them all - for a
// list with hundreds/thousands of objects (pods, jobs), rebuilding `items.value` on every
// single one of those is O(n^2) work purely on page load. Coalescing same-burst events
// into one rebuild fixes that without changing the backend's per-object event shape.
const FLUSH_DELAY_MS = 30;

/**
 * Live list of a resource kind for the currently connected context, kept up to date via
 * a backend watch. Re-subscribes whenever the context, kind, or namespace changes.
 */
export function useResourceList(
  kind: MaybeRefOrGetter<string>,
  namespace?: Ref<string | undefined>,
  fieldSelector?: Ref<string | undefined>,
  labelSelector?: Ref<string | undefined>,
) {
  const cluster = useClusterStore();
  const items = ref<any[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  let stopWatching: (() => void) | null = null;
  let generation = 0;
  let flushTimer: ReturnType<typeof setTimeout> | null = null;

  async function subscribe() {
    const myGeneration = ++generation;
    stopWatching?.();
    stopWatching = null;
    if (flushTimer !== null) {
      clearTimeout(flushTimer);
      flushTimer = null;
    }

    const contextName = cluster.currentContext;
    if (!contextName) {
      items.value = [];
      return;
    }

    const ns = namespace?.value;
    const fieldSel = fieldSelector?.value;
    const labelSel = labelSelector?.value;
    loading.value = true;
    error.value = null;

    try {
      const kindValue = toValue(kind);
      const initial = await listResources(contextName, kindValue, ns, fieldSel, labelSel);
      if (myGeneration !== generation) return;

      const byKey = new Map<string, any>(initial.map((obj) => [objectKey(obj), obj]));
      items.value = Array.from(byKey.values());

      function scheduleFlush() {
        if (flushTimer !== null) return;
        flushTimer = setTimeout(() => {
          flushTimer = null;
          if (myGeneration !== generation) return;
          items.value = Array.from(byKey.values());
        }, FLUSH_DELAY_MS);
      }

      const unlisten = await startWatch(
        contextName,
        kindValue,
        ns,
        (event: ResourceEvent) => {
          if (myGeneration !== generation) return;
          const key = objectKey(event.object);
          if (event.type === "upsert") {
            byKey.set(key, event.object);
          } else {
            byKey.delete(key);
          }
          scheduleFlush();
        },
        fieldSel,
        labelSel,
      );

      if (myGeneration !== generation) {
        unlisten();
      } else {
        stopWatching = unlisten;
      }
    } catch (e) {
      if (myGeneration === generation) error.value = String(e);
    } finally {
      if (myGeneration === generation) loading.value = false;
    }
  }

  watch(
    () => [
      cluster.currentContext,
      toValue(kind),
      namespace?.value,
      fieldSelector?.value,
      labelSelector?.value,
    ],
    subscribe,
    { immediate: true },
  );

  onScopeDispose(() => {
    generation++;
    stopWatching?.();
    if (flushTimer !== null) {
      clearTimeout(flushTimer);
      flushTimer = null;
    }
  });

  return { items, loading, error };
}

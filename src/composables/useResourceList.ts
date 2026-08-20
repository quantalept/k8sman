import { ref, watch, onScopeDispose, toValue, type MaybeRefOrGetter, type Ref } from "vue";
import { listResources, startWatch, type ResourceEvent } from "../api/resources";
import { useClusterStore } from "../stores/cluster";

function objectKey(obj: any): string {
  const meta = obj?.metadata ?? {};
  return meta.uid ?? `${meta.namespace ?? ""}/${meta.name ?? ""}`;
}

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

  async function subscribe() {
    const myGeneration = ++generation;
    stopWatching?.();
    stopWatching = null;

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
          items.value = Array.from(byKey.values());
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
  });

  return { items, loading, error };
}

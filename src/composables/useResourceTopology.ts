import { ref, watch, type Ref } from "vue";
import { getResource, listResources } from "../api/resources";
import { useClusterStore } from "../stores/cluster";
import { getParentRefs, OWNED_CHILD_KINDS, type ResourceRef } from "../topology/relationships";

export interface TopologyGraphNode extends ResourceRef {
  id: string;
  focal?: boolean;
}

export interface TopologyGraphEdge {
  id: string;
  source: string;
  target: string;
  label: string;
}

function nodeId(ref: ResourceRef): string {
  return `${ref.kind}/${ref.namespace ?? ""}/${ref.name}`;
}

/**
 * The focal resource's direct relationships only (one hop in each direction), not a
 * whole-namespace graph - `recenter` re-runs this on a neighbor so a user walks the graph
 * hop by hop instead of ever fetching more than a small neighborhood at once.
 */
export function useResourceTopology(focal: Ref<ResourceRef>) {
  const nodes = ref<TopologyGraphNode[]>([]);
  const edges = ref<TopologyGraphEdge[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const cluster = useClusterStore();

  async function load() {
    const contextName = cluster.currentContext;
    if (!contextName) return;

    const focalRef = focal.value;
    const focalId = nodeId(focalRef);
    const newNodes = new Map<string, TopologyGraphNode>();
    const newEdges: TopologyGraphEdge[] = [];
    newNodes.set(focalId, { ...focalRef, id: focalId, focal: true });

    function addNode(ref: ResourceRef): string {
      const id = nodeId(ref);
      if (!newNodes.has(id)) newNodes.set(id, { ...ref, id });
      return id;
    }

    loading.value = true;
    error.value = null;
    try {
      const obj = await getResource(contextName, focalRef.kind, focalRef.namespace, focalRef.name);

      for (const parent of getParentRefs(focalRef.kind, obj)) {
        const id = addNode(parent);
        newEdges.push({ id: `${id}->${focalId}:${parent.label}`, source: id, target: focalId, label: parent.label });
      }

      const uid = obj?.metadata?.uid;
      for (const childKind of OWNED_CHILD_KINDS[focalRef.kind] ?? []) {
        try {
          const children = await listResources(contextName, childKind, focalRef.namespace);
          for (const child of children) {
            const owned = (child.metadata?.ownerReferences ?? []).some((o: any) => o.uid === uid);
            if (!owned) continue;
            const id = addNode({
              kind: childKind,
              namespace: child.metadata?.namespace,
              name: child.metadata?.name,
            });
            newEdges.push({ id: `${focalId}->${id}:owns`, source: focalId, target: id, label: "Owns" });
          }
        } catch {
          // Kind might not exist/be listable on this cluster - skip it rather than
          // failing the whole graph over one missing child kind.
        }
      }

      if (focalRef.kind === "Service") {
        const selector = obj?.spec?.selector ?? {};
        const parts = Object.entries(selector).map(([k, v]) => `${k}=${v}`);
        if (parts.length > 0) {
          const pods = await listResources(
            contextName,
            "Pod",
            focalRef.namespace,
            undefined,
            parts.join(","),
          );
          for (const pod of pods) {
            const id = addNode({
              kind: "Pod",
              namespace: pod.metadata?.namespace,
              name: pod.metadata?.name,
            });
            newEdges.push({ id: `${focalId}->${id}:selects`, source: focalId, target: id, label: "Selects" });
          }
        }
      }

      if (focalRef.kind === "Node") {
        const pods = await listResources(
          contextName,
          "Pod",
          undefined,
          `spec.nodeName=${focalRef.name}`,
        );
        for (const pod of pods) {
          const id = addNode({ kind: "Pod", namespace: pod.metadata?.namespace, name: pod.metadata?.name });
          newEdges.push({ id: `${focalId}->${id}:runs`, source: focalId, target: id, label: "Runs" });
        }
      }

      nodes.value = Array.from(newNodes.values());
      edges.value = newEdges;
    } catch (e) {
      error.value = String(e);
      nodes.value = [{ ...focalRef, id: focalId, focal: true }];
      edges.value = [];
    } finally {
      loading.value = false;
    }
  }

  watch([focal, () => cluster.currentContext], load, { immediate: true });

  function recenter(ref: ResourceRef) {
    focal.value = { kind: ref.kind, namespace: ref.namespace, name: ref.name };
  }

  return { nodes, edges, loading, error, recenter, reload: load };
}

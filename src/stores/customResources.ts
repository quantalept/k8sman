import { defineStore } from "pinia";
import { listResourceKinds, type ResourceKindRef } from "../api/resources";
import { KNOWN_BUILTIN_GROUPS } from "../resourceKinds";
import { useClusterStore } from "./cluster";

export interface CustomResourceGroup {
  group: string;
  kinds: ResourceKindRef[];
}

export const useCustomResourcesStore = defineStore("customResources", {
  state: () => ({
    groups: [] as CustomResourceGroup[],
    loading: false,
    error: null as string | null,
  }),
  actions: {
    async load() {
      const cluster = useClusterStore();
      const contextName = cluster.currentContext;
      if (!contextName) {
        this.groups = [];
        return;
      }
      this.loading = true;
      this.error = null;
      try {
        const kinds = await listResourceKinds(contextName);
        const custom = kinds.filter((k) => !KNOWN_BUILTIN_GROUPS.has(k.group));

        const byGroup = new Map<string, ResourceKindRef[]>();
        for (const k of custom) {
          const list = byGroup.get(k.group) ?? [];
          list.push(k);
          byGroup.set(k.group, list);
        }

        this.groups = Array.from(byGroup.entries())
          .map(([group, groupKinds]) => ({
            group,
            kinds: [...groupKinds].sort((a, b) => a.kind.localeCompare(b.kind)),
          }))
          .sort((a, b) => a.group.localeCompare(b.group));
      } catch (e) {
        this.error = String(e);
      } finally {
        this.loading = false;
      }
    },
  },
});

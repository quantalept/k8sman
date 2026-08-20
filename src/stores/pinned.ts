import { defineStore } from "pinia";
import { getUiState, setUiState } from "../api/uiState";

export interface PinnedResource {
  kind: string;
  namespace?: string;
  name: string;
  visitedAt?: number;
}

const KEY_PINNED = "pinnedResources";
const KEY_RECENT = "recentResources";
const MAX_RECENT = 20;

function sameResource(a: PinnedResource, b: PinnedResource): boolean {
  return a.kind === b.kind && a.namespace === b.namespace && a.name === b.name;
}

export const usePinnedStore = defineStore("pinned", {
  state: () => ({
    pinned: [] as PinnedResource[],
    recent: [] as PinnedResource[],
    loaded: false,
  }),
  actions: {
    async load() {
      if (this.loaded) return;
      const [pinned, recent] = await Promise.all([
        getUiState<PinnedResource[]>(KEY_PINNED),
        getUiState<PinnedResource[]>(KEY_RECENT),
      ]);
      this.pinned = pinned ?? [];
      this.recent = recent ?? [];
      this.loaded = true;
    },

    isPinned(resource: PinnedResource): boolean {
      return this.pinned.some((p) => sameResource(p, resource));
    },

    async pin(resource: PinnedResource) {
      if (this.isPinned(resource)) return;
      this.pinned = [...this.pinned, resource];
      await setUiState(KEY_PINNED, this.pinned);
    },

    async unpin(resource: PinnedResource) {
      this.pinned = this.pinned.filter((p) => !sameResource(p, resource));
      await setUiState(KEY_PINNED, this.pinned);
    },

    async recordVisit(resource: PinnedResource) {
      const withoutDup = this.recent.filter((r) => !sameResource(r, resource));
      this.recent = [{ ...resource, visitedAt: Date.now() }, ...withoutDup].slice(0, MAX_RECENT);
      await setUiState(KEY_RECENT, this.recent);
    },
  },
});

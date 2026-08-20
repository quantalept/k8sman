import { defineStore } from "pinia";
import { getUiState, setUiState } from "../api/uiState";

export interface SavedView {
  id: string;
  name: string;
  /** The K8s Kind, e.g. "Pod" or "Deployment". */
  kind: string;
  /** Sidebar route segment for this kind, e.g. "pods" or "deployments". */
  route: string;
  namespace?: string;
  labelSelector?: string;
}

const KEY_VIEWS = "savedViews";

export const useSavedViewsStore = defineStore("savedViews", {
  state: () => ({
    views: [] as SavedView[],
    loaded: false,
  }),
  actions: {
    async load() {
      if (this.loaded) return;
      this.views = (await getUiState<SavedView[]>(KEY_VIEWS)) ?? [];
      this.loaded = true;
    },

    async save(view: Omit<SavedView, "id">) {
      const saved: SavedView = { ...view, id: crypto.randomUUID() };
      this.views = [...this.views, saved];
      await setUiState(KEY_VIEWS, this.views);
      return saved;
    },

    async remove(id: string) {
      this.views = this.views.filter((v) => v.id !== id);
      await setUiState(KEY_VIEWS, this.views);
    },
  },
});

import { defineStore } from "pinia";

export interface ContextInfo {
  name: string;
  cluster: string;
  namespace?: string;
}

export const useClusterStore = defineStore("cluster", {
  state: () => ({
    contexts: [] as ContextInfo[],
    currentContext: null as string | null,
    connected: false,
  }),
  actions: {
    // Wired up to `list_contexts` / `switch_context` Tauri commands in the
    // cluster-configuration build-order step.
  },
});

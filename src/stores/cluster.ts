import { defineStore } from "pinia";
import * as clusterApi from "../api/cluster";

export interface ContextInfo {
  name: string;
  cluster: string;
  namespace?: string;
  source: string;
}

export const useClusterStore = defineStore("cluster", {
  state: () => ({
    contexts: [] as ContextInfo[],
    kubeconfigPaths: [] as string[],
    currentContext: null as string | null,
    connecting: false,
    error: null as string | null,
  }),
  getters: {
    connected: (state) => state.currentContext !== null,
  },
  actions: {
    async loadContexts() {
      this.error = null;
      try {
        this.contexts = await clusterApi.listContexts();
        this.kubeconfigPaths = await clusterApi.listKubeconfigPaths();
      } catch (e) {
        this.error = String(e);
      }
    },

    async restoreLastContext() {
      const last = await clusterApi.currentContext();
      if (last) {
        await this.switchContext(last);
      }
    },

    async switchContext(name: string) {
      this.connecting = true;
      this.error = null;
      try {
        await clusterApi.switchContext(name);
        this.currentContext = name;
      } catch (e) {
        this.error = String(e);
        throw e;
      } finally {
        this.connecting = false;
      }
    },

    async addKubeconfig(path: string) {
      this.error = null;
      try {
        this.contexts = await clusterApi.addKubeconfig(path);
        this.kubeconfigPaths = await clusterApi.listKubeconfigPaths();
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },

    async removeKubeconfig(path: string) {
      this.error = null;
      try {
        this.contexts = await clusterApi.removeKubeconfig(path);
        this.kubeconfigPaths = await clusterApi.listKubeconfigPaths();
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },
  },
});

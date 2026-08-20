import { defineStore } from "pinia";
import * as clusterApi from "../api/cluster";
import { getUiState, setUiState } from "../api/uiState";

export interface ContextInfo {
  name: string;
  cluster: string;
  namespace?: string;
  source: string;
}

const KEY_CONNECTED = "connectedContexts";
const KEY_ACTIVE = "activeContext";

export const useClusterStore = defineStore("cluster", {
  state: () => ({
    contexts: [] as ContextInfo[],
    kubeconfigPaths: [] as string[],
    /** Open cluster tabs. */
    connectedContexts: [] as string[],
    /** The tab currently being viewed. */
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

    async persistTabs() {
      await setUiState(KEY_CONNECTED, this.connectedContexts);
      await setUiState(KEY_ACTIVE, this.currentContext);
    },

    /** Restores previously open tabs on app boot. Falls back to the last single context
     * used before multi-cluster tabs existed. */
    async restoreTabs() {
      const [connected, active] = await Promise.all([
        getUiState<string[]>(KEY_CONNECTED),
        getUiState<string | null>(KEY_ACTIVE),
      ]);

      if (connected && connected.length > 0) {
        for (const name of connected) {
          try {
            await this.connectTab(name, { activate: name === active });
          } catch {
            // A saved tab's context may no longer exist (kubeconfig changed); skip it.
          }
        }
        return;
      }

      const legacy = await clusterApi.currentContext();
      if (legacy) {
        await this.connectTab(legacy);
      }
    },

    /** Connects (if needed) and opens a tab for the given context, making it active. */
    async connectTab(name: string, opts: { activate?: boolean } = { activate: true }) {
      this.connecting = true;
      this.error = null;
      try {
        await clusterApi.switchContext(name);
        if (!this.connectedContexts.includes(name)) {
          this.connectedContexts.push(name);
        }
        if (opts.activate !== false) {
          this.currentContext = name;
        }
        await this.persistTabs();
      } catch (e) {
        this.error = String(e);
        throw e;
      } finally {
        this.connecting = false;
      }
    },

    /** Switches to an already-open tab - no backend call, since it's already connected. */
    activateTab(name: string) {
      if (!this.connectedContexts.includes(name)) return;
      this.currentContext = name;
      void this.persistTabs();
    },

    closeTab(name: string) {
      this.connectedContexts = this.connectedContexts.filter((c) => c !== name);
      if (this.currentContext === name) {
        this.currentContext = this.connectedContexts[0] ?? null;
      }
      void this.persistTabs();
    },

    /** @deprecated use connectTab - kept as an alias for existing call sites. */
    async switchContext(name: string) {
      await this.connectTab(name);
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

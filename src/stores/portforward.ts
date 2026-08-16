import { defineStore } from "pinia";
import * as pfApi from "../api/portforward";
import type { PortForwardInfo } from "../api/portforward";

interface ActiveForward extends PortForwardInfo {
  status: "active" | "error";
  message?: string;
}

export const usePortForwardStore = defineStore("portforward", {
  state: () => ({
    forwards: [] as ActiveForward[],
    error: null as string | null,
  }),
  actions: {
    async start(contextName: string, namespace: string, pod: string, remotePort: number, localPort?: number) {
      this.error = null;
      try {
        const info = await pfApi.startPortForward(contextName, namespace, pod, remotePort, localPort);
        const forward: ActiveForward = { ...info, status: "active" };
        this.forwards.push(forward);

        await pfApi.onPortForwardStatus(info.forwardId, (message) => {
          const entry = this.forwards.find((f) => f.forwardId === info.forwardId);
          if (entry) {
            entry.status = "error";
            entry.message = message;
          }
        });

        return forward;
      } catch (e) {
        this.error = String(e);
        throw e;
      }
    },

    async stop(forwardId: string) {
      await pfApi.stopPortForward(forwardId);
      this.forwards = this.forwards.filter((f) => f.forwardId !== forwardId);
    },
  },
});

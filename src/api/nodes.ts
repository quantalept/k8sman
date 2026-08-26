import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { stopWatch } from "./resources";

export function cordonNode(
  contextName: string,
  name: string,
  unschedulable: boolean,
): Promise<any> {
  return invoke("cordon_node", { contextName, name, unschedulable });
}

export function taintNode(
  contextName: string,
  name: string,
  key: string,
  value: string | undefined,
  effect: string,
): Promise<any> {
  return invoke("taint_node", { contextName, name, key, value, effect });
}

export function untaintNode(
  contextName: string,
  name: string,
  key: string,
  effect: string,
): Promise<any> {
  return invoke("untaint_node", { contextName, name, key, effect });
}

export type NodeDrainEvent =
  | { state: "skipped-daemonset"; pod: string }
  | { state: "skipped-mirror"; pod: string }
  | { state: "evicting"; pod: string }
  | { state: "evicted"; pod: string }
  | { state: "error"; pod: string; message: string }
  | { state: "done" };

export function startNodeDrain(
  contextName: string,
  name: string,
  onEvent: (event: NodeDrainEvent) => void,
): Promise<UnlistenFn> {
  return invoke<string>("start_node_drain", { contextName, name }).then((streamId) =>
    listen<NodeDrainEvent>(`node-drain-event:${streamId}`, (e) => onEvent(e.payload)).then(
      (unlisten) => () => {
        unlisten();
        void stopWatch(streamId);
      },
    ),
  );
}

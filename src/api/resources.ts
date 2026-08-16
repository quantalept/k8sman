import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ResourceKindRef {
  group: string;
  version: string;
  kind: string;
  namespaced: boolean;
}

export type ResourceEvent =
  | { type: "upsert"; object: any }
  | { type: "delete"; object: any };

export function listResourceKinds(contextName: string): Promise<ResourceKindRef[]> {
  return invoke("list_resource_kinds", { contextName });
}

export function listResources(
  contextName: string,
  kind: string,
  namespace?: string,
): Promise<any[]> {
  return invoke("list_resources", { contextName, kind, namespace });
}

export function startWatch(
  contextName: string,
  kind: string,
  namespace: string | undefined,
  onEvent: (event: ResourceEvent) => void,
): Promise<UnlistenFn> {
  return invoke<string>("start_watch", { contextName, kind, namespace }).then((streamId) =>
    listen<ResourceEvent>(`resource-event:${streamId}`, (e) => onEvent(e.payload)).then(
      (unlisten) => () => {
        unlisten();
        void stopWatch(streamId);
      },
    ),
  );
}

export function stopWatch(streamId: string): Promise<void> {
  return invoke("stop_watch", { streamId });
}

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ResourceKindRef {
  group: string;
  version: string;
  kind: string;
  plural: string;
  namespaced: boolean;
}

export type ResourceEvent =
  | { type: "upsert"; object: any }
  | { type: "delete"; object: any };

export function listResourceKinds(contextName: string): Promise<ResourceKindRef[]> {
  return invoke("list_resource_kinds", { contextName });
}

export interface PrinterColumn {
  name: string;
  type: string;
  jsonPath: string;
}

export function getCrdPrinterColumns(
  contextName: string,
  group: string,
  plural: string,
  version: string,
): Promise<PrinterColumn[]> {
  return invoke("get_crd_printer_columns", { contextName, group, plural, version });
}

export function listResources(
  contextName: string,
  kind: string,
  namespace?: string,
  fieldSelector?: string,
  labelSelector?: string,
): Promise<any[]> {
  return invoke("list_resources", { contextName, kind, namespace, fieldSelector, labelSelector });
}

export function getResource(
  contextName: string,
  kind: string,
  namespace: string | undefined,
  name: string,
): Promise<any> {
  return invoke("get_resource", { contextName, kind, namespace, name });
}

export function deleteResource(
  contextName: string,
  kind: string,
  namespace: string | undefined,
  name: string,
): Promise<void> {
  return invoke("delete_resource", { contextName, kind, namespace, name });
}

export function applyResource(contextName: string, yaml: string): Promise<any> {
  return invoke("apply_resource", { contextName, yaml });
}

export function scaleResource(
  contextName: string,
  kind: string,
  namespace: string | undefined,
  name: string,
  replicas: number,
): Promise<any> {
  return invoke("scale_resource", { contextName, kind, namespace, name, replicas });
}

export function restartRollout(
  contextName: string,
  kind: string,
  namespace: string | undefined,
  name: string,
): Promise<any> {
  return invoke("restart_rollout", { contextName, kind, namespace, name });
}

export function startWatch(
  contextName: string,
  kind: string,
  namespace: string | undefined,
  onEvent: (event: ResourceEvent) => void,
  fieldSelector?: string,
  labelSelector?: string,
): Promise<UnlistenFn> {
  return invoke<string>("start_watch", {
    contextName,
    kind,
    namespace,
    fieldSelector,
    labelSelector,
  }).then(
    (streamId) =>
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

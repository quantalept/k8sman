import { invoke } from "@tauri-apps/api/core";

export interface DebugPodRef {
  namespace: string;
  name: string;
}

export function createNodeDebugPod(
  contextName: string,
  nodeName: string,
  image?: string,
): Promise<DebugPodRef> {
  return invoke("create_node_debug_pod", { contextName, nodeName, image });
}

export function addEphemeralContainer(
  contextName: string,
  namespace: string,
  pod: string,
  containerName: string,
  image: string,
  targetContainer?: string,
): Promise<any> {
  return invoke("add_ephemeral_container", {
    contextName,
    namespace,
    pod,
    containerName,
    image,
    targetContainer,
  });
}

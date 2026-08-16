import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface PortForwardInfo {
  forwardId: string;
  namespace: string;
  pod: string;
  localPort: number;
  remotePort: number;
}

export function startPortForward(
  contextName: string,
  namespace: string,
  pod: string,
  remotePort: number,
  localPort?: number,
): Promise<PortForwardInfo> {
  return invoke("start_port_forward", { contextName, namespace, pod, remotePort, localPort });
}

export function stopPortForward(forwardId: string): Promise<void> {
  return invoke("stop_port_forward", { forwardId });
}

export function onPortForwardStatus(
  forwardId: string,
  onMessage: (message: string) => void,
): Promise<UnlistenFn> {
  return listen<string>(`portforward-status:${forwardId}`, (e) => onMessage(e.payload));
}

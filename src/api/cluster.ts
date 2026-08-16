import { invoke } from "@tauri-apps/api/core";
import type { ContextInfo } from "../stores/cluster";

export function listContexts(): Promise<ContextInfo[]> {
  return invoke("list_contexts");
}

export function listKubeconfigPaths(): Promise<string[]> {
  return invoke("list_kubeconfig_paths");
}

export function addKubeconfig(path: string): Promise<ContextInfo[]> {
  return invoke("add_kubeconfig", { path });
}

export function removeKubeconfig(path: string): Promise<ContextInfo[]> {
  return invoke("remove_kubeconfig", { path });
}

export function currentContext(): Promise<string | null> {
  return invoke("current_context");
}

export function switchContext(contextName: string): Promise<ContextInfo> {
  return invoke("switch_context", { contextName });
}

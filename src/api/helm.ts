import { invoke } from "@tauri-apps/api/core";

export interface HelmReleaseSummary {
  name: string;
  namespace: string;
  revision: number;
  status: string;
  chart: string;
  chartVersion: string;
  appVersion: string;
  updated: string;
  description: string;
}

export function listHelmReleases(
  contextName: string,
  namespace?: string,
): Promise<HelmReleaseSummary[]> {
  return invoke("list_helm_releases", { contextName, namespace });
}

export function getHelmRelease(
  contextName: string,
  namespace: string,
  name: string,
  revision?: number,
): Promise<any> {
  return invoke("get_helm_release", { contextName, namespace, name, revision });
}

export function listHelmReleaseHistory(
  contextName: string,
  namespace: string,
  name: string,
): Promise<HelmReleaseSummary[]> {
  return invoke("list_helm_release_history", { contextName, namespace, name });
}

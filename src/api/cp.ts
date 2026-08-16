import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface CpProgress {
  sent: number;
  total: number;
}

export interface CpTarget {
  contextName: string;
  namespace: string;
  pod: string;
  container?: string;
}

export async function cpToPod(
  target: CpTarget,
  localPath: string,
  remoteDir: string,
  onProgress?: (p: CpProgress) => void,
): Promise<void> {
  const transferId = crypto.randomUUID();
  const unlisten = onProgress
    ? await listen<CpProgress>(`cp-progress:${transferId}`, (e) => onProgress(e.payload))
    : null;
  try {
    await invoke("cp_to_pod", { ...target, localPath, remoteDir, transferId });
  } finally {
    unlisten?.();
  }
}

export async function cpFromPod(
  target: CpTarget,
  remotePath: string,
  localDir: string,
  onProgress?: (p: CpProgress) => void,
): Promise<void> {
  const transferId = crypto.randomUUID();
  const unlisten = onProgress
    ? await listen<CpProgress>(`cp-progress:${transferId}`, (e) => onProgress(e.payload))
    : null;
  try {
    await invoke("cp_from_pod", { ...target, remotePath, localDir, transferId });
  } finally {
    unlisten?.();
  }
}

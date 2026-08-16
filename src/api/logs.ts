import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface StartLogStreamOptions {
  contextName: string;
  namespace: string;
  pod: string;
  container?: string;
  follow?: boolean;
  tailLines?: number;
  previous?: boolean;
}

export async function startLogStream(
  opts: StartLogStreamOptions,
  onLine: (line: string) => void,
  onDone?: () => void,
): Promise<UnlistenFn> {
  const streamId = await invoke<string>("start_log_stream", {
    contextName: opts.contextName,
    namespace: opts.namespace,
    pod: opts.pod,
    container: opts.container,
    follow: opts.follow ?? true,
    tailLines: opts.tailLines,
    previous: opts.previous ?? false,
  });

  const unlistenLine = await listen<string>(`log:${streamId}`, (e) => onLine(e.payload));
  const unlistenDone = onDone
    ? await listen(`log-done:${streamId}`, () => onDone())
    : () => {};

  return () => {
    unlistenLine();
    unlistenDone();
    void stopLogStream(streamId);
  };
}

export function stopLogStream(streamId: string): Promise<void> {
  return invoke("stop_log_stream", { streamId });
}

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface StartExecOptions {
  contextName: string;
  namespace: string;
  pod: string;
  container?: string;
  command: string[];
}

export interface ExecHandle {
  sessionId: string;
  write: (data: string) => void;
  resize: (cols: number, rows: number) => void;
  dispose: () => void;
}

export async function startExec(
  opts: StartExecOptions,
  onOutput: (chunk: string) => void,
  onDone?: () => void,
): Promise<ExecHandle> {
  const sessionId = await invoke<string>("start_exec", {
    contextName: opts.contextName,
    namespace: opts.namespace,
    pod: opts.pod,
    container: opts.container,
    command: opts.command,
  });

  const unlistenOutput = await listen<string>(`exec-output:${sessionId}`, (e) =>
    onOutput(e.payload),
  );
  const unlistenDone: UnlistenFn = onDone
    ? await listen(`exec-done:${sessionId}`, () => onDone())
    : () => {};

  return {
    sessionId,
    write: (data: string) => {
      void invoke("exec_write", { sessionId, data });
    },
    resize: (cols: number, rows: number) => {
      void invoke("exec_resize", { sessionId, cols, rows });
    },
    dispose: () => {
      unlistenOutput();
      unlistenDone();
      void invoke("stop_exec", { sessionId });
    },
  };
}

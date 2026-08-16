<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { startExec, type ExecHandle } from "../api/exec";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  namespace: string;
  pod: string;
  container?: string;
  command?: string[];
}>();

const cluster = useClusterStore();
const containerEl = ref<HTMLDivElement | null>(null);

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let handle: ExecHandle | null = null;
let inputDisposable: { dispose(): void } | null = null;
let resizeObserver: ResizeObserver | null = null;

async function connect() {
  handle?.dispose();
  handle = null;
  term?.clear();

  const contextName = cluster.currentContext;
  if (!contextName || !term) return;

  term.writeln("\x1b[2mconnecting…\x1b[0m");

  handle = await startExec(
    {
      contextName,
      namespace: props.namespace,
      pod: props.pod,
      container: props.container,
      command: props.command ?? ["/bin/sh"],
    },
    (chunk) => term?.write(chunk),
    () => term?.writeln("\r\n\x1b[2m[session ended]\x1b[0m"),
  );

  handle.resize(term.cols, term.rows);
}

onMounted(() => {
  if (!containerEl.value) return;
  term = new Terminal({
    convertEol: true,
    cursorBlink: true,
    fontSize: 13,
    theme: { background: "#101014" },
  });
  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(containerEl.value);
  fitAddon.fit();

  inputDisposable = term.onData((data) => handle?.write(data));

  resizeObserver = new ResizeObserver(() => {
    fitAddon?.fit();
    if (term) handle?.resize(term.cols, term.rows);
  });
  resizeObserver.observe(containerEl.value);

  connect();
});

watch(() => [cluster.currentContext, props.namespace, props.pod, props.container], connect);

onBeforeUnmount(() => {
  handle?.dispose();
  inputDisposable?.dispose();
  resizeObserver?.disconnect();
  term?.dispose();
});
</script>

<template>
  <div ref="containerEl" class="exec-terminal" />
</template>

<style scoped>
.exec-terminal {
  height: 400px;
  padding: 8px;
  background: #101014;
  border-radius: 6px;
}
</style>

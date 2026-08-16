<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { startLogStream } from "../api/logs";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  namespace: string;
  pod: string;
  container?: string;
  follow?: boolean;
  tailLines?: number;
}>();

const cluster = useClusterStore();
const containerEl = ref<HTMLDivElement | null>(null);

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let stopStream: (() => void) | null = null;
let resizeObserver: ResizeObserver | null = null;

async function subscribe() {
  stopStream?.();
  stopStream = null;
  term?.clear();

  const contextName = cluster.currentContext;
  if (!contextName || !term) return;

  stopStream = await startLogStream(
    {
      contextName,
      namespace: props.namespace,
      pod: props.pod,
      container: props.container,
      follow: props.follow ?? true,
      tailLines: props.tailLines ?? 200,
    },
    (line) => term?.writeln(line),
    () => term?.writeln("\x1b[2m[stream ended]\x1b[0m"),
  );
}

onMounted(() => {
  if (!containerEl.value) return;
  term = new Terminal({
    convertEol: true,
    disableStdin: true,
    fontSize: 13,
    theme: { background: "#101014" },
  });
  fitAddon = new FitAddon();
  term.loadAddon(fitAddon);
  term.open(containerEl.value);
  fitAddon.fit();

  resizeObserver = new ResizeObserver(() => fitAddon?.fit());
  resizeObserver.observe(containerEl.value);

  subscribe();
});

watch(() => [cluster.currentContext, props.namespace, props.pod, props.container], subscribe);

onBeforeUnmount(() => {
  stopStream?.();
  resizeObserver?.disconnect();
  term?.dispose();
});
</script>

<template>
  <div ref="containerEl" class="log-viewer" />
</template>

<style scoped>
.log-viewer {
  height: 400px;
  padding: 8px;
  background: #101014;
  border-radius: 6px;
}
</style>

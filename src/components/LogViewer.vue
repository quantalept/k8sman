<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { NSpin } from "naive-ui";
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
const loading = ref(false);
const error = ref<string | null>(null);

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let stopStream: (() => void) | null = null;
let resizeObserver: ResizeObserver | null = null;

async function subscribe() {
  stopStream?.();
  stopStream = null;
  term?.clear();
  error.value = null;

  const contextName = cluster.currentContext;
  if (!contextName || !term) return;

  loading.value = true;
  try {
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
    loading.value = false;
  } catch (e) {
    loading.value = false;
    error.value = String(e);
  }
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
  <n-spin :show="loading" description="Loading logs…" class="log-viewer-wrap">
    <div v-if="error" class="log-viewer-error">{{ error }}</div>
    <div ref="containerEl" class="log-viewer" />
  </n-spin>
</template>

<style scoped>
.log-viewer-wrap {
  display: block;
}
.log-viewer {
  height: 400px;
  padding: 8px;
  background: #101014;
  border-radius: 6px;
}
.log-viewer-error {
  color: #e88080;
  padding: 8px 0;
}
</style>

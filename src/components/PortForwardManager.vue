<script setup lang="ts">
import { ref, computed, h } from "vue";
import { NSpace, NInputNumber, NInput, NButton, NDataTable, NAlert, NTag, type DataTableColumns } from "naive-ui";
import { usePortForwardStore } from "../stores/portforward";
import { useClusterStore } from "../stores/cluster";

const props = defineProps<{
  /** Pre-fill and lock the namespace/pod fields, e.g. when embedded in a pod detail page. */
  namespace?: string;
  pod?: string;
}>();

const cluster = useClusterStore();
const store = usePortForwardStore();

const namespace = ref(props.namespace ?? "");
const pod = ref(props.pod ?? "");
const remotePort = ref<number | null>(null);
const localPort = ref<number | null>(null);
const starting = ref(false);

const visibleForwards = computed(() =>
  props.pod
    ? store.forwards.filter((f) => f.namespace === props.namespace && f.pod === props.pod)
    : store.forwards,
);

async function start() {
  const contextName = cluster.currentContext;
  if (!contextName || !namespace.value || !pod.value || !remotePort.value) return;
  starting.value = true;
  try {
    await store.start(contextName, namespace.value, pod.value, remotePort.value, localPort.value ?? undefined);
    remotePort.value = null;
    localPort.value = null;
  } finally {
    starting.value = false;
  }
}

const columns: DataTableColumns<any> = [
  { title: "Namespace", key: "namespace" },
  { title: "Pod", key: "pod" },
  { title: "Local", key: "localPort", render: (row) => `127.0.0.1:${row.localPort}` },
  { title: "Remote port", key: "remotePort" },
  {
    title: "Status",
    key: "status",
    render: (row) =>
      row.status === "active"
        ? h(NTag, { type: "success", size: "small", round: true }, { default: () => "active" })
        : h(NTag, { type: "error", size: "small", round: true }, { default: () => row.message ?? "error" }),
  },
  {
    title: "",
    key: "actions",
    render: (row) =>
      h(
        NButton,
        { size: "tiny", tertiary: true, type: "error", onClick: () => store.stop(row.forwardId) },
        { default: () => "Stop" },
      ),
  },
];
</script>

<template>
  <n-space vertical>
    <n-alert v-if="store.error" type="error" :title="store.error" closable />
    <n-space align="center">
      <n-input v-if="!props.namespace" v-model:value="namespace" placeholder="namespace" style="width: 140px" />
      <n-input v-if="!props.pod" v-model:value="pod" placeholder="pod name" style="width: 200px" />
      <n-input-number v-model:value="remotePort" placeholder="remote port" style="width: 140px" :min="1" :max="65535" />
      <n-input-number v-model:value="localPort" placeholder="local port (auto)" style="width: 160px" :min="1" :max="65535" />
      <n-button type="primary" :loading="starting" :disabled="!remotePort" @click="start">Forward</n-button>
    </n-space>
    <n-data-table :columns="columns" :data="visibleForwards" :bordered="false" size="small" />
  </n-space>
</template>

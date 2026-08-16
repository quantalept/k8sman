<script setup lang="ts">
import { h, ref, computed } from "vue";
import { useRouter } from "vue-router";
import { NCard, NTag, NSelect, NSpace, NText } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";
import { useResourceList } from "../composables/useResourceList";
import { usePodMetrics } from "../composables/usePodMetrics";
import { formatCores, formatBytes } from "../api/metrics";

const router = useRouter();
const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
const { usage: podUsage, supported: metricsSupported } = usePodMetrics(effectiveNamespace);
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

function podStatus(row: any): { text: string; type: "success" | "warning" | "error" | "default" } {
  const phase = row.status?.phase ?? "Unknown";
  if (phase === "Running") return { text: phase, type: "success" };
  if (phase === "Pending") return { text: phase, type: "warning" };
  if (phase === "Failed") return { text: phase, type: "error" };
  return { text: phase, type: "default" };
}

function restarts(row: any): number {
  const statuses: any[] = row.status?.containerStatuses ?? [];
  return statuses.reduce((sum, s) => sum + (s.restartCount ?? 0), 0);
}

const columns = [
  {
    title: "Status",
    key: "status",
    render: (row: any) => {
      const s = podStatus(row);
      return h(NTag, { type: s.type, size: "small", round: true }, { default: () => s.text });
    },
  },
  { title: "Restarts", key: "restarts", render: (row: any) => restarts(row) },
  {
    title: "CPU",
    key: "cpu",
    render: (row: any) => {
      const usage = podUsage.value.get(`${row.metadata.namespace}/${row.metadata.name}`);
      return usage ? formatCores(usage.cpuCores) : "-";
    },
  },
  {
    title: "Memory",
    key: "memory",
    render: (row: any) => {
      const usage = podUsage.value.get(`${row.metadata.namespace}/${row.metadata.name}`);
      return usage ? formatBytes(usage.memoryBytes) : "-";
    },
  },
];

function onRowProps(row: any) {
  return {
    style: "cursor: pointer",
    onClick: () => router.push(`/pods/${row.metadata.namespace}/${row.metadata.name}`),
  };
}
</script>

<template>
  <n-card title="Pods">
    <n-space vertical>
      <n-select
        v-model:value="namespace"
        :options="namespaceOptions"
        style="width: 240px"
        placeholder="All namespaces"
      />
      <n-text v-if="!metricsSupported" depth="3">
        Metrics unavailable - is metrics-server installed on this cluster?
      </n-text>
      <ResourceTable kind="Pod" :namespace="effectiveNamespace" :columns="columns" :row-props="onRowProps" />
    </n-space>
  </n-card>
</template>

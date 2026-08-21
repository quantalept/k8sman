<script setup lang="ts">
import { h, ref, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NCard, NTag, NSelect, NSpace, NText, NInput } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";
import SavedViewControls from "../components/SavedViewControls.vue";
import { useResourceList } from "../composables/useResourceList";
import { usePodMetrics } from "../composables/usePodMetrics";
import { formatCores, formatBytes } from "../api/metrics";
import { looksLikeLabelSelector } from "../labelSelector";

const route = useRoute();
const router = useRouter();
const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);
const labelSelector = ref<string>("");
const trimmedFilter = computed(() => labelSelector.value.trim());
const isSelectorSyntax = computed(() => looksLikeLabelSelector(trimmedFilter.value));
// The same input does double duty: `key=value` syntax narrows via the API as a real label
// selector, anything else is a client-side substring search on the resource name.
const effectiveLabelSelector = computed(() =>
  isSelectorSyntax.value ? trimmedFilter.value : undefined,
);
const effectiveSearch = computed(() => (isSelectorSyntax.value ? undefined : trimmedFilter.value));

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

function cpuCores(row: any): number {
  return podUsage.value.get(`${row.metadata.namespace}/${row.metadata.name}`)?.cpuCores ?? -1;
}

function memoryBytes(row: any): number {
  return podUsage.value.get(`${row.metadata.namespace}/${row.metadata.name}`)?.memoryBytes ?? -1;
}

const columns = [
  {
    title: "Status",
    key: "status",
    render: (row: any) => {
      const s = podStatus(row);
      return h(NTag, { type: s.type, size: "small", round: true }, { default: () => s.text });
    },
    sorter: (a: any, b: any) => podStatus(a).text.localeCompare(podStatus(b).text),
  },
  {
    title: "Restarts",
    key: "restarts",
    render: (row: any) => restarts(row),
    sorter: (a: any, b: any) => restarts(a) - restarts(b),
  },
  {
    title: "CPU",
    key: "cpu",
    render: (row: any) => {
      const usage = podUsage.value.get(`${row.metadata.namespace}/${row.metadata.name}`);
      return usage ? formatCores(usage.cpuCores) : "-";
    },
    sorter: (a: any, b: any) => cpuCores(a) - cpuCores(b),
  },
  {
    title: "Memory",
    key: "memory",
    render: (row: any) => {
      const usage = podUsage.value.get(`${row.metadata.namespace}/${row.metadata.name}`);
      return usage ? formatBytes(usage.memoryBytes) : "-";
    },
    sorter: (a: any, b: any) => memoryBytes(a) - memoryBytes(b),
  },
];

// Seed filters from ?ns=&labels= when navigating here (e.g. from the command palette's
// saved-view entries).
watch(
  () => route.query,
  () => {
    const ns = route.query.ns;
    const labels = route.query.labels;
    if (typeof ns === "string") namespace.value = ns;
    if (typeof labels === "string") labelSelector.value = labels;
  },
  { immediate: true },
);

function applySavedView(view: { namespace?: string; labelSelector?: string }) {
  namespace.value = view.namespace ?? "";
  labelSelector.value = view.labelSelector ?? "";
}

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
      <n-space align="center">
        <n-select
          v-model:value="namespace"
          :options="namespaceOptions"
          style="width: 240px"
          placeholder="All namespaces"
        />
        <n-input
          v-model:value="labelSelector"
          placeholder="Search by name, or label selector (app=foo,tier=bar)"
          style="width: 320px"
          clearable
        />
      </n-space>
      <SavedViewControls
        kind="Pod"
        route="pods"
        :namespace="effectiveNamespace"
        :label-selector="effectiveLabelSelector"
        @apply="applySavedView"
      />
      <n-text v-if="!metricsSupported" depth="3">
        Metrics unavailable - is metrics-server installed on this cluster?
      </n-text>
      <ResourceTable
        kind="Pod"
        :namespace="effectiveNamespace"
        :label-selector="effectiveLabelSelector"
        :search="effectiveSearch"
        :columns="columns"
        :row-props="onRowProps"
      />
    </n-space>
  </n-card>
</template>

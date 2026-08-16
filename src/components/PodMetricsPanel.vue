<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { NGrid, NGridItem, NAlert, NText, NDataTable, type DataTableColumns } from "naive-ui";
import MetricCard from "./MetricCard.vue";
import { usePodMetrics, type ContainerUsage } from "../composables/usePodMetrics";
import { formatCores, formatBytes } from "../api/metrics";

const props = defineProps<{
  namespace: string;
  pod: string;
}>();

const MAX_SAMPLES = 30;

interface Sample {
  timestamp: number;
  cpuCores: number;
  memoryBytes: number;
}

const namespaceRef = computed(() => props.namespace);
const { usage, error, supported } = usePodMetrics(namespaceRef);

const samples = ref<Sample[]>([]);

watch(usage, (map) => {
  const entry = map.get(`${props.namespace}/${props.pod}`);
  if (!entry) return;
  samples.value = [
    ...samples.value.slice(-(MAX_SAMPLES - 1)),
    { timestamp: Date.now(), cpuCores: entry.cpuCores, memoryBytes: entry.memoryBytes },
  ];
});

const timestamps = computed(() => samples.value.map((s) => Math.floor(s.timestamp / 1000)));
const cpuSeries = computed<[number[], number[]]>(() => [
  timestamps.value,
  samples.value.map((s) => s.cpuCores * 1000),
]);
const memorySeries = computed<[number[], number[]]>(() => [
  timestamps.value,
  samples.value.map((s) => s.memoryBytes / 1024 / 1024),
]);

const latest = computed(() => samples.value[samples.value.length - 1]);
const cpuLabel = computed(() => (latest.value ? formatCores(latest.value.cpuCores) : "-"));
const memoryLabel = computed(() => (latest.value ? formatBytes(latest.value.memoryBytes) : "-"));

const containers = computed<ContainerUsage[]>(
  () => usage.value.get(`${props.namespace}/${props.pod}`)?.containers ?? [],
);

const containerColumns: DataTableColumns<ContainerUsage> = [
  { title: "Container", key: "name" },
  { title: "CPU", key: "cpu", render: (row) => formatCores(row.cpuCores) },
  { title: "Memory", key: "memory", render: (row) => formatBytes(row.memoryBytes) },
];
</script>

<template>
  <n-alert v-if="!supported" type="warning" title="Metrics unavailable">
    Could not reach the metrics API - is metrics-server installed on this cluster?
    <div v-if="error" style="margin-top: 4px"><n-text depth="3">{{ error }}</n-text></div>
  </n-alert>
  <div v-else>
    <n-grid :cols="2" :x-gap="16" style="margin-bottom: 16px">
      <n-grid-item>
        <MetricCard title="CPU" :value="cpuLabel" color="#3987e5" :series="cpuSeries" />
      </n-grid-item>
      <n-grid-item>
        <MetricCard title="Memory" :value="memoryLabel" color="#199e70" :series="memorySeries" />
      </n-grid-item>
    </n-grid>
    <n-data-table :columns="containerColumns" :data="containers" :bordered="false" size="small" />
  </div>
</template>

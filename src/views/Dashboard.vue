<script setup lang="ts">
import { computed } from "vue";
import { NGrid, NGridItem, NAlert, NText } from "naive-ui";
import MetricCard from "../components/MetricCard.vue";
import { useClusterMetrics } from "../composables/useClusterMetrics";
import { useClusterStore } from "../stores/cluster";

const cluster = useClusterStore();
const { samples, error, supported } = useClusterMetrics();

const timestamps = computed(() => samples.value.map((s) => Math.floor(s.timestamp / 1000)));

const cpuSeries = computed<[number[], number[]]>(() => [
  timestamps.value,
  samples.value.map((s) => s.cpuPercent),
]);
const memorySeries = computed<[number[], number[]]>(() => [
  timestamps.value,
  samples.value.map((s) => s.memoryPercent),
]);

const latest = computed(() => samples.value[samples.value.length - 1]);
const cpuLabel = computed(() => (latest.value ? `${Math.round(latest.value.cpuPercent)}%` : "-"));
const memoryLabel = computed(() =>
  latest.value ? `${Math.round(latest.value.memoryPercent)}%` : "-",
);
</script>

<template>
  <div v-if="!cluster.connected">
    <n-text depth="3">Connect to a cluster to see metrics.</n-text>
  </div>
  <n-alert v-else-if="!supported" type="warning" title="Metrics unavailable">
    Could not reach the metrics API - is metrics-server installed on this cluster?
    <div v-if="error" style="margin-top: 4px"><n-text depth="3">{{ error }}</n-text></div>
  </n-alert>
  <n-grid v-else :cols="2" :x-gap="16">
    <n-grid-item>
      <MetricCard title="CPU Usage" :value="cpuLabel" color="#3987e5" :series="cpuSeries" />
    </n-grid-item>
    <n-grid-item>
      <MetricCard title="Memory Usage" :value="memoryLabel" color="#199e70" :series="memorySeries" />
    </n-grid-item>
  </n-grid>
</template>

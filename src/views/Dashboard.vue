<script setup lang="ts">
import { computed, onMounted } from "vue";
import { NGrid, NGridItem, NAlert, NText, NCard, NSpace, NTag, NEmpty } from "naive-ui";
import MetricCard from "../components/MetricCard.vue";
import { useClusterMetrics } from "../composables/useClusterMetrics";
import { useClusterStore } from "../stores/cluster";
import { usePinnedStore, type PinnedResource } from "../stores/pinned";

const cluster = useClusterStore();
const pinned = usePinnedStore();
const { samples, error, supported } = useClusterMetrics();

onMounted(() => pinned.load());

function resourcePath(r: PinnedResource): string {
  if (r.kind === "Pod") return `/pods/${r.namespace}/${r.name}`;
  if (r.kind === "Node") return `/nodes/${r.name}`;
  const query = r.namespace ? `?ns=${encodeURIComponent(r.namespace)}` : "";
  return `/resources/${r.kind}/${r.name}${query}`;
}

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

  <n-card title="Pinned & Recent" style="margin-top: 16px">
    <n-empty v-if="pinned.pinned.length === 0 && pinned.recent.length === 0" description="Nothing pinned or viewed yet" />
    <n-space v-else vertical :size="6">
      <n-space v-for="r in pinned.pinned" :key="`pinned-${r.kind}-${r.namespace}-${r.name}`" align="center">
        <n-tag type="warning" size="small" round>pinned</n-tag>
        <router-link :to="resourcePath(r)">{{ r.kind }}/{{ r.name }}</router-link>
        <n-text v-if="r.namespace" depth="3">({{ r.namespace }})</n-text>
      </n-space>
      <n-space
        v-for="r in pinned.recent.filter((r) => !pinned.isPinned(r))"
        :key="`recent-${r.kind}-${r.namespace}-${r.name}`"
        align="center"
      >
        <n-tag size="small" round>recent</n-tag>
        <router-link :to="resourcePath(r)">{{ r.kind }}/{{ r.name }}</router-link>
        <n-text v-if="r.namespace" depth="3">({{ r.namespace }})</n-text>
      </n-space>
    </n-space>
  </n-card>
</template>

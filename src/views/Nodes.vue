<script setup lang="ts">
import { NCard, NTag } from "naive-ui";
import { h } from "vue";
import ResourceTable from "../components/ResourceTable.vue";

function isReady(row: any): boolean {
  const conditions: any[] = row.status?.conditions ?? [];
  return conditions.some((c) => c.type === "Ready" && c.status === "True");
}

const columns = [
  {
    title: "Status",
    key: "status",
    render: (row: any) =>
      h(
        NTag,
        { type: isReady(row) ? "success" : "error", size: "small", round: true },
        { default: () => (isReady(row) ? "Ready" : "Not Ready") },
      ),
  },
  {
    title: "Version",
    key: "version",
    render: (row: any) => row.status?.nodeInfo?.kubeletVersion ?? "-",
  },
];
</script>

<template>
  <n-card title="Nodes">
    <ResourceTable kind="Node" :show-namespace="false" :columns="columns" />
  </n-card>
</template>

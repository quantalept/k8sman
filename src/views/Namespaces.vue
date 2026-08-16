<script setup lang="ts">
import { NCard } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";

const columns = [
  {
    title: "Status",
    key: "status",
    render: (row: any) => row.status?.phase ?? "-",
  },
];
</script>

<template>
  <n-card title="Namespaces">
    <ResourceTable kind="Namespace" :show-namespace="false" :columns="columns" />
  </n-card>
</template>

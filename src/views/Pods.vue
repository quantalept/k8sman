<script setup lang="ts">
import { h, ref, computed } from "vue";
import { useRouter } from "vue-router";
import { NCard, NTag, NSelect, NSpace } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";
import { useResourceList } from "../composables/useResourceList";

const router = useRouter();
const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
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
      <ResourceTable kind="Pod" :namespace="effectiveNamespace" :columns="columns" :row-props="onRowProps" />
    </n-space>
  </n-card>
</template>

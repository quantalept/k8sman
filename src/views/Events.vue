<script setup lang="ts">
import { ref, computed } from "vue";
import { NCard, NSpace, NSelect, NDataTable, NAlert, type DataTableColumns } from "naive-ui";
import { useResourceList } from "../composables/useResourceList";

const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

const { items, loading, error } = useResourceList("Event", effectiveNamespace);

const sorted = computed(() =>
  [...items.value].sort((a, b) => {
    const at = a.lastTimestamp ?? a.eventTime ?? a.metadata?.creationTimestamp ?? "";
    const bt = b.lastTimestamp ?? b.eventTime ?? b.metadata?.creationTimestamp ?? "";
    return String(bt).localeCompare(String(at));
  }),
);

const columns: DataTableColumns<any> = [
  { title: "Type", key: "type", render: (row) => row.type ?? "-" },
  { title: "Reason", key: "reason", render: (row) => row.reason ?? "-" },
  {
    title: "Object",
    key: "object",
    render: (row) =>
      `${row.involvedObject?.kind ?? "-"}/${row.involvedObject?.name ?? "-"}`,
  },
  { title: "Message", key: "message", render: (row) => row.message ?? "-" },
  {
    title: "Last Seen",
    key: "lastSeen",
    render: (row) => row.lastTimestamp ?? row.eventTime ?? "-",
  },
];
</script>

<template>
  <n-card title="Events">
    <n-space vertical>
      <n-select
        v-model:value="namespace"
        :options="namespaceOptions"
        style="width: 240px"
        placeholder="All namespaces"
      />
      <n-alert v-if="error" type="error" :title="error" closable />
      <n-data-table :columns="columns" :data="sorted" :loading="loading" :bordered="false" size="small" />
    </n-space>
  </n-card>
</template>

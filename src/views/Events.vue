<script setup lang="ts">
import { ref, computed } from "vue";
import { NCard, NSpace, NSelect, NDataTable, NAlert, type DataTableColumns } from "naive-ui";
import { useResourceList } from "../composables/useResourceList";
import { PAGE_TABLE_MAX_HEIGHT } from "../layout";

const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

const { items, loading, error } = useResourceList("Event", effectiveNamespace);

function lastSeen(row: any): string {
  return row.lastTimestamp ?? row.eventTime ?? row.metadata?.creationTimestamp ?? "";
}

const columns: DataTableColumns<any> = [
  {
    title: "Type",
    key: "type",
    render: (row) => row.type ?? "-",
    sorter: (a: any, b: any) => (a.type ?? "").localeCompare(b.type ?? ""),
  },
  {
    title: "Reason",
    key: "reason",
    render: (row) => row.reason ?? "-",
    sorter: (a: any, b: any) => (a.reason ?? "").localeCompare(b.reason ?? ""),
  },
  {
    title: "Object",
    key: "object",
    render: (row) => `${row.involvedObject?.kind ?? "-"}/${row.involvedObject?.name ?? "-"}`,
    sorter: (a: any, b: any) =>
      (a.involvedObject?.name ?? "").localeCompare(b.involvedObject?.name ?? ""),
  },
  { title: "Message", key: "message", render: (row) => row.message ?? "-" },
  {
    title: "Last Seen",
    key: "lastSeen",
    render: (row) => lastSeen(row) || "-",
    sorter: (a: any, b: any) => lastSeen(b).localeCompare(lastSeen(a)),
    defaultSortOrder: "ascend",
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
      <n-data-table
        :columns="columns"
        :data="items"
        :loading="loading"
        :max-height="PAGE_TABLE_MAX_HEIGHT"
        :bordered="false"
        size="small"
      />
    </n-space>
  </n-card>
</template>

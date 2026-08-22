<script setup lang="ts">
import { computed } from "vue";
import { NDataTable, NAlert, type DataTableColumns } from "naive-ui";
import { useResourceList } from "../composables/useResourceList";
import { TAB_TABLE_MAX_HEIGHT } from "../layout";

const props = defineProps<{
  namespace?: string;
  involvedObjectName: string;
}>();

const namespaceRef = computed(() => props.namespace);
const fieldSelectorRef = computed(() => {
  const parts = [`involvedObject.name=${props.involvedObjectName}`];
  if (props.namespace) parts.push(`involvedObject.namespace=${props.namespace}`);
  return parts.join(",");
});

const { items, loading, error } = useResourceList("Event", namespaceRef, fieldSelectorRef);

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
  <n-alert v-if="error" type="error" :title="error" closable style="margin-bottom: 12px" />
  <n-data-table
    :columns="columns"
    :data="items"
    :loading="loading"
    :max-height="TAB_TABLE_MAX_HEIGHT"
    :bordered="false"
    size="small"
  />
</template>

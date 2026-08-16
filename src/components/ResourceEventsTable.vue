<script setup lang="ts">
import { computed } from "vue";
import { NDataTable, NAlert, type DataTableColumns } from "naive-ui";
import { useResourceList } from "../composables/useResourceList";

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
  { title: "Message", key: "message", render: (row) => row.message ?? "-" },
  {
    title: "Last Seen",
    key: "lastSeen",
    render: (row) => row.lastTimestamp ?? row.eventTime ?? "-",
  },
];
</script>

<template>
  <n-alert v-if="error" type="error" :title="error" closable style="margin-bottom: 12px" />
  <n-data-table :columns="columns" :data="sorted" :loading="loading" :bordered="false" size="small" />
</template>

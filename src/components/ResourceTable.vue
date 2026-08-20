<script setup lang="ts">
import { computed, toRef } from "vue";
import { NDataTable, NAlert, NText, type DataTableColumns } from "naive-ui";
import { useResourceList } from "../composables/useResourceList";

const props = defineProps<{
  kind: string;
  namespace?: string;
  labelSelector?: string;
  /** Extra columns beyond the built-in Name / Namespace / Age. */
  columns?: DataTableColumns<any>;
  showNamespace?: boolean;
  rowProps?: (row: any) => Record<string, unknown>;
}>();

const namespaceRef = toRef(props, "namespace");
const kindRef = toRef(props, "kind");
const labelSelectorRef = toRef(props, "labelSelector");
const { items, loading, error } = useResourceList(kindRef, namespaceRef, undefined, labelSelectorRef);

function age(obj: any): string {
  const ts = obj?.metadata?.creationTimestamp;
  if (!ts) return "-";
  const ms = Date.now() - new Date(ts).getTime();
  const minutes = Math.floor(ms / 60000);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h`;
  return `${Math.floor(hours / 24)}d`;
}

const columns = computed<DataTableColumns<any>>(() => {
  const base: DataTableColumns<any> = [
    { title: "Name", key: "metadata.name", render: (row) => row.metadata?.name ?? "" },
  ];
  if (props.showNamespace !== false) {
    base.push({
      title: "Namespace",
      key: "metadata.namespace",
      render: (row) => row.metadata?.namespace ?? "-",
    });
  }
  base.push(...(props.columns ?? []));
  base.push({ title: "Age", key: "age", render: (row) => age(row) });
  return base;
});

const rowKey = (row: any) => row.metadata?.uid ?? `${row.metadata?.namespace}/${row.metadata?.name}`;
</script>

<template>
  <div>
    <n-alert v-if="error" type="error" :title="error" closable style="margin-bottom: 12px" />
    <n-data-table
      :columns="columns"
      :data="items"
      :loading="loading"
      :row-key="rowKey"
      :row-props="rowProps"
      :bordered="false"
      size="small"
    />
    <n-text v-if="!loading && items.length === 0 && !error" depth="3">No resources found.</n-text>
  </div>
</template>

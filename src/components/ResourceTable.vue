<script setup lang="ts">
import { computed, toRef } from "vue";
import { NDataTable, NAlert, NText, type DataTableColumns } from "naive-ui";
import { useResourceList } from "../composables/useResourceList";
import { PAGE_TABLE_MAX_HEIGHT } from "../layout";

const props = defineProps<{
  kind: string;
  namespace?: string;
  fieldSelector?: string;
  labelSelector?: string;
  /** Client-side substring filter on the resource name. */
  search?: string;
  /** Extra columns beyond the built-in Name / Namespace / Age. */
  columns?: DataTableColumns<any>;
  showNamespace?: boolean;
  rowProps?: (row: any) => Record<string, unknown>;
  /** Override when this table sits somewhere with more/less chrome above it (e.g. a tab). */
  maxHeight?: string | number;
}>();

const namespaceRef = toRef(props, "namespace");
const kindRef = toRef(props, "kind");
const fieldSelectorRef = toRef(props, "fieldSelector");
const labelSelectorRef = toRef(props, "labelSelector");
const { items, loading, error } = useResourceList(
  kindRef,
  namespaceRef,
  fieldSelectorRef,
  labelSelectorRef,
);

const filteredItems = computed(() => {
  const term = props.search?.trim().toLowerCase();
  if (!term) return items.value;
  return items.value.filter((obj: any) => obj.metadata?.name?.toLowerCase().includes(term));
});

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

function creationEpoch(obj: any): number {
  const ts = obj?.metadata?.creationTimestamp;
  return ts ? new Date(ts).getTime() : 0;
}

const columns = computed<DataTableColumns<any>>(() => {
  const base: DataTableColumns<any> = [
    {
      title: "Name",
      key: "metadata.name",
      minWidth: 140,
      ellipsis: { tooltip: true },
      render: (row) => row.metadata?.name ?? "",
      sorter: (a: any, b: any) => (a.metadata?.name ?? "").localeCompare(b.metadata?.name ?? ""),
    },
  ];
  if (props.showNamespace !== false) {
    base.push({
      title: "Namespace",
      key: "metadata.namespace",
      minWidth: 100,
      render: (row) => row.metadata?.namespace ?? "-",
      sorter: (a: any, b: any) =>
        (a.metadata?.namespace ?? "").localeCompare(b.metadata?.namespace ?? ""),
    });
  }
  base.push(...(props.columns ?? []));
  base.push({
    title: "Age",
    key: "age",
    minWidth: 70,
    render: (row) => age(row),
    // Newest first by default - sorting by the rendered "3d"/"10m" string would be wrong
    // (string-sorts "10m" before "3d"), so sort by the real timestamp instead.
    sorter: (a: any, b: any) => creationEpoch(b) - creationEpoch(a),
  });
  return base;
});

const rowKey = (row: any) => row.metadata?.uid ?? `${row.metadata?.namespace}/${row.metadata?.name}`;
</script>

<template>
  <div>
    <n-alert v-if="error" type="error" :title="error" closable style="margin-bottom: 12px" />
    <n-data-table
      :columns="columns"
      :data="filteredItems"
      :loading="loading"
      :row-key="rowKey"
      :row-props="rowProps"
      :max-height="maxHeight ?? PAGE_TABLE_MAX_HEIGHT"
      :bordered="false"
      size="small"
    />
    <n-text v-if="!loading && filteredItems.length === 0 && !error" depth="3">No resources found.</n-text>
  </div>
</template>

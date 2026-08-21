<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NCard, NSelect, NSpace, NText, type DataTableColumns } from "naive-ui";
import ResourceTable from "../components/ResourceTable.vue";
import { useResourceList } from "../composables/useResourceList";
import { useCustomResourcesStore } from "../stores/customResources";
import { useClusterStore } from "../stores/cluster";
import { getCrdPrinterColumns, type PrinterColumn } from "../api/resources";
import { evalPrinterColumnPath } from "../jsonPath";

const route = useRoute();
const router = useRouter();
const customResources = useCustomResourcesStore();
const cluster = useClusterStore();

const group = computed(() => String(route.params.group));
const version = computed(() => String(route.params.version));
const kind = computed(() => String(route.params.kind));

const kindRef = computed(() =>
  customResources.groups
    .flatMap((g) => g.kinds)
    .find((k) => k.group === group.value && k.version === version.value && k.kind === kind.value),
);

const namespace = ref<string>("");
const effectiveNamespace = computed(() => namespace.value || undefined);

const { items: namespaces } = useResourceList("Namespace");
const namespaceOptions = computed(() => [
  { label: "All namespaces", value: "" },
  ...namespaces.value.map((ns: any) => ({ label: ns.metadata.name, value: ns.metadata.name })),
]);

const printerColumns = ref<PrinterColumn[]>([]);

async function loadPrinterColumns() {
  const contextName = cluster.currentContext;
  const k = kindRef.value;
  if (!contextName || !k) {
    printerColumns.value = [];
    return;
  }
  try {
    printerColumns.value = await getCrdPrinterColumns(contextName, k.group, k.plural, k.version);
  } catch {
    printerColumns.value = [];
  }
}

watch([kindRef, () => cluster.currentContext], loadPrinterColumns, { immediate: true });

const columns = computed<DataTableColumns<any>>(() =>
  printerColumns.value
    // ResourceTable already appends its own Age column from metadata.creationTimestamp.
    .filter((c) => c.name !== "Age")
    .map((c) => ({
      title: c.name,
      key: c.name,
      render: (row: any) => {
        const value = evalPrinterColumnPath(row, c.jsonPath);
        return value === undefined || value === null ? "-" : String(value);
      },
    })),
);

function onRowProps(row: any) {
  return {
    style: "cursor: pointer",
    onClick: () => {
      const query = row.metadata?.namespace ? { ns: row.metadata.namespace } : {};
      router.push({ path: `/resources/${kind.value}/${row.metadata.name}`, query });
    },
  };
}
</script>

<template>
  <n-card v-if="kindRef" :title="kindRef.kind">
    <n-space vertical>
      <n-select
        v-if="kindRef.namespaced"
        v-model:value="namespace"
        :options="namespaceOptions"
        style="width: 240px"
        placeholder="All namespaces"
      />
      <ResourceTable
        :kind="kindRef.kind"
        :namespace="kindRef.namespaced ? effectiveNamespace : undefined"
        :show-namespace="kindRef.namespaced"
        :columns="columns"
        :row-props="onRowProps"
      />
    </n-space>
  </n-card>
  <n-text v-else depth="3">Unknown custom resource. Try reopening it from the sidebar.</n-text>
</template>
